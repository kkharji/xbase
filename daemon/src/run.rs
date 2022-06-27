mod bin;
mod handler;
mod service;
mod simulator;

use crate::constants::State;
use crate::constants::DAEMON_STATE;
use crate::device::Device;
use crate::logger::Logger;
use crate::util::log_request;
use crate::Result;
use process_stream::Process;
use std::sync::Arc;
use tokio::sync::MutexGuard;
use xbase_proto::LoggingTask;
use xbase_proto::{BuildSettings, Client, RunRequest};

pub use service::RunService;
pub use {bin::*, simulator::*};

#[async_trait::async_trait]
pub trait Runner {
    /// Run Project
    async fn run<'a>(&self, logger: &Logger) -> Result<Process>;
}

/// Handle RunRequest
pub async fn handle(req: RunRequest) -> Result<LoggingTask> {
    let client = &req.client;
    let sep = log_request!("Run", client, req);

    let logger = Logger::new(
        format!("Run {}", client.root.display()),
        format!("run_{}.log", client.abbrev_root().replace("/", "_")),
        client.root.as_path(),
    )
    .await?;

    let ref key = req.to_string();
    let state = DAEMON_STATE.clone();
    let ref mut state = state.lock().await;

    if req.ops.is_once() {
        // TODO(run): might want to keep track of ran services
        RunService::new(state, req).await?;
        return Ok(Default::default());
    }

    let client = req.client.clone();
    if req.ops.is_watch() {
        let watcher = state.watcher.get(&req.client.root)?;
        if watcher.contains_key(key) {
            logger.append(format!("Already watching with {key}!!"));
        } else {
            let pid = req.client.pid.to_owned();
            let run_service = RunService::new(state, req).await?;
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

    Ok(Default::default())
}

async fn get_runner<'a>(
    state: &'a MutexGuard<'_, State>,
    client: &Client,
    settings: &BuildSettings,
    device: Option<&Device>,
    is_once: bool,
    logger: &Arc<Logger>,
) -> Result<process_stream::Process> {
    let root = &client.root;
    let target = &settings.target;
    let (runner, args) = state
        .projects
        .get(root)?
        .get_runner(&settings, device, logger)?;

    logger.append(format!("Building {target}"));
    log::info!("[target: {target}] building .....");

    // TODO: ensure that the build process was successfully before trying this once
    // if !success {
    //     let msg = format!("[target: {target}] failed to be built",);
    //     logger.nvim.echo_err(&msg).await?;
    //     log::error!("[target: {target}] failed to be built");
    //     log::error!("[ran: 'xcodebuild {}']", args.join(" "));
    //     return Err(Error::Build(msg));
    // } else {
    //     log::info!("[target: {target}] built successfully");
    // }
    let process = runner.run(logger).await?;
    log::info!("[target: {target}] running .....");

    Ok(process)
}
