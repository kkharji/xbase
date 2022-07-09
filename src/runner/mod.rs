mod bin;
mod device;
mod handler;
mod service;
mod simulator;

use crate::*;
use async_trait::async_trait;
use process_stream::Process;
use std::sync::Arc;
use tokio::sync::OwnedMutexGuard;

pub use service::RunService;
pub use {bin::*, device::*, service::*, simulator::*};

#[async_trait]
pub trait Runner {
    async fn run<'a>(&self, broadcast: &Broadcast) -> Result<Process>;
}

pub async fn get_runner<'a>(
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
        broadcast.log_error(format!("[{target}] xcodebuild {}", args.join(" ")));
        broadcast.open_logger();
        return Err(crate::Error::Run(msg));
    }

    let process = runner.run(broadcast).await?;

    broadcast.update_statusline(StatuslineState::Running);

    Ok(process)
}
