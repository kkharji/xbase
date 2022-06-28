mod bin;
mod handler;
mod service;
mod simulator;

use crate::broadcast::Broadcast;
use crate::constants::State;
use crate::constants::DAEMON_STATE;
use crate::device::Device;
use crate::util::log_request;
use crate::Result;
use process_stream::Process;
use std::sync::Arc;
use tokio::sync::MutexGuard;
use xbase_proto::{BuildSettings, Client, RunRequest};

pub use service::RunService;
pub use {bin::*, simulator::*};

#[async_trait::async_trait]
pub trait Runner {
    /// Run Project
    async fn run<'a>(&self, logger: &Broadcast) -> Result<Process>;
}

/// Handle RunRequest
/// TODO: Watch runners
pub async fn handle(req: RunRequest) -> Result<()> {
    let client = &req.client;
    let root = &client.root;
    let sep = log_request!("Run", root, req);

    let ref key = req.to_string();
    let state = DAEMON_STATE.clone();
    let ref mut state = state.lock().await;
    let logger = state.broadcasters.get_or_init(root).await?;
    let logger = logger.upgrade().unwrap();

    if req.ops.is_once() {
        // TODO(run): might want to keep track of ran services
        RunService::new(state, req, &logger).await?;
        return Ok(Default::default());
    }

    let client = req.client.clone();
    if req.ops.is_watch() {
        let watcher = state.watcher.get(&req.client.root)?;
        if watcher.contains_key(key) {
            logger.info(format!("Already watching with {key}!!"))?;
        } else {
            let run_service = RunService::new(state, req, &logger).await?;
            let watcher = state.watcher.get_mut(&client.root)?;
            watcher.add(run_service)?;
        }
    } else {
        log::info!("[target: {}] stopping .....", &req.settings.target);
        let watcher = state.watcher.get_mut(&req.client.root)?;
        let listener = watcher.remove(&req.to_string())?;
        listener.discard(state).await?;
    }

    log::info!("{sep}",);
    log::info!("{sep}",);

    Ok(())
}

async fn get_runner<'a>(
    state: &'a MutexGuard<'_, State>,
    client: &Client,
    settings: &BuildSettings,
    device: Option<&Device>,
    _is_once: bool,
    logger: &Arc<Broadcast>,
) -> Result<process_stream::Process> {
    let root = &client.root;
    let target = &settings.target;
    let project = state.projects.get(root)?;
    let (runner, _) = project.get_runner(&settings, device, logger)?;

    log::info!("[target: {target}] running .....");

    Ok(runner.run(logger).await?)
}
