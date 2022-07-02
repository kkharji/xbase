mod bin;
mod handler;
mod service;
mod simulator;

use crate::broadcast::Broadcast;
use crate::device::Device;
use crate::project::ProjectImplementer;
use crate::store::TryGetDaemonObject;
use crate::Result;
use process_stream::Process;
use std::sync::Arc;
use tokio::sync::OwnedMutexGuard;
use xbase_proto::{BuildSettings, RunRequest, StatuslineState};

pub use service::RunService;
pub use {bin::*, simulator::*};

#[async_trait::async_trait]
pub trait Runner {
    /// Run Project
    async fn run<'a>(&self, broadcast: &Broadcast) -> Result<Process>;
}

/// Handle RunRequest
/// TODO: Watch runners
pub async fn handle(req: RunRequest) -> Result<()> {
    let root = req.root.clone();

    log::trace!("{:#?}", req);

    let ref key = req.to_string();
    let broadcast = root.try_get_broadcast().await?;
    let mut project = root.try_get_project().await?;

    let watcher = req.root.try_get_mutex_watcher().await?;
    let weak_watcher = Arc::downgrade(&watcher);

    if req.ops.is_once() {
        // TODO(run): might want to keep track of ran services
        RunService::new(&mut project, req, &broadcast, weak_watcher).await?;

        return Ok(Default::default());
    }

    let mut watcher = watcher.lock().await;

    if req.ops.is_watch() {
        broadcast.update_statusline(StatuslineState::Watching);
        if watcher.contains_key(key) {
            broadcast.warn(format!("Already watching with {key}!!"));
        } else {
            let run_service = RunService::new(&mut project, req, &broadcast, weak_watcher).await?;
            watcher.add(run_service)?;
        }
    } else {
        let listener = watcher.remove(&req.to_string())?;
        listener.discard().await?;
        broadcast.info(format!("[{}] Watcher Stopped", &req.settings.target));
        broadcast.update_statusline(StatuslineState::Clear);
    }

    Ok(())
}

async fn get_runner<'a>(
    project: &mut OwnedMutexGuard<ProjectImplementer>,
    settings: &BuildSettings,
    device: Option<&Device>,
    _is_once: bool,
    broadcast: &Arc<Broadcast>,
) -> Result<Process> {
    let target = &settings.target;
    let device_name = device.map(|d| d.to_string()).unwrap_or("macOs".into());

    broadcast.info(format!("[{target}({device_name})] Running ⚙"));

    let (runner, args, mut recv) = project.get_runner(&settings, device, broadcast)?;

    broadcast.update_statusline(StatuslineState::Processing);

    if !recv.recv().await.unwrap_or_default() {
        let msg = format!("[{target}] Failed to build for running ");
        broadcast.error(&msg);
        broadcast.log_error(format!("xcodebuild {}", args.join(" ")));
        broadcast.open_logger();
        return Err(crate::Error::Run(msg));
    }

    let process = runner.run(broadcast).await?;

    broadcast.update_statusline(StatuslineState::Running);

    Ok(process)
}
