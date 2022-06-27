use crate::constants::DAEMON_STATE;
use crate::state::State;
use crate::util::log_request;
use crate::watch::{Event, Watchable};
use crate::Result;
use async_trait::async_trait;
use tokio::sync::MutexGuard;
use xbase_proto::{BuildRequest, LoggingTask};

/// Handle build Request
pub async fn handle(req: BuildRequest) -> Result<LoggingTask> {
    let state = DAEMON_STATE.clone();
    let ref mut state = state.lock().await;
    let client = &req.client;

    log_request!("Build", client, req);

    if req.ops.is_once() {
        req.trigger(state, &Event::default()).await?;
        return Ok(LoggingTask::default());
    }

    if req.ops.is_watch() {
        state
            .clients
            .get(&req.client.pid)?
            .set_watching(true)
            .await?;
        state.watcher.get_mut(&req.client.root)?.add(req)?;
    } else {
        state
            .clients
            .get(&req.client.pid)?
            .set_watching(false)
            .await?;
        state
            .watcher
            .get_mut(&req.client.root)?
            .remove(&req.to_string())?;
    }

    state.sync_client_state().await?;

    Ok(LoggingTask::default())
}

#[async_trait]
impl Watchable for BuildRequest {
    async fn trigger(&self, state: &MutexGuard<State>, _event: &Event) -> Result<()> {
        let is_once = self.ops.is_once();
        let (root, config) = (&self.client.root, &self.settings);
        let (stream, args) = state.projects.get(root)?.build(&config, None)?;
        let nvim = state.clients.get(&self.client.pid)?;
        let logger = &mut nvim.logger();

        logger.set_title(format!(
            "{}:{}",
            if is_once { "Build" } else { "Rebuild" },
            config.target
        ));

        log::info!("[target: {}] building .....", self.settings.target);
        let success = logger.consume_build_logs(stream, false, !is_once).await?;
        if !success {
            let ref msg = format!("Failed: {} ", config.to_string());
            nvim.echo_err(msg).await?;
            log::error!("[target: {}] failed to be built", self.settings.target);
            log::error!("[ran: 'xcodebuild {}']", args.join(" "));
        } else {
            log::info!("[target: {}] built successfully", self.settings.target);
        };

        Ok(())
    }

    /// A function that controls whether a a Watchable should restart
    async fn should_trigger(&self, _state: &MutexGuard<State>, event: &Event) -> bool {
        event.is_content_update_event()
            || event.is_rename_event()
            || event.is_create_event()
            || event.is_remove_event()
            || !(event.path().exists() || event.is_seen())
    }

    /// A function that controls whether a watchable should be droped
    async fn should_discard(&self, _state: &MutexGuard<State>, _event: &Event) -> bool {
        false
    }

    /// Drop watchable for watching a given file system
    async fn discard(&self, _state: &MutexGuard<State>) -> Result<()> {
        Ok(())
    }
}
