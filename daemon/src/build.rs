use crate::broadcast::Broadcast;
use crate::constants::{State, DAEMON_STATE};
use crate::watch::{Event, Watchable};
use crate::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::MutexGuard;
use xbase_proto::{BuildRequest, StatuslineState};

/// Handle build Request
pub async fn handle(req: BuildRequest) -> Result<()> {
    let state = DAEMON_STATE.clone();
    let ref mut state = state.lock().await;
    let broadcast = state.broadcasters.get(&req.root)?;
    let target = &req.settings.target;
    let args = &req.settings.to_string();

    log::trace!("{:#?}", req);

    if req.ops.is_once() {
        req.trigger(state, &Event::default(), &broadcast).await?;
        return Ok(());
    }

    if req.ops.is_watch() {
        broadcast.info(format!("[{target}] Watching  with '{args}'"));
        broadcast.update_statusline(StatuslineState::Watching);
        state.watcher.get_mut(&req.root)?.add(req)?;
    } else {
        broadcast.info(format!("[{}] Wathcer Stopped", &req.settings.target));
        state.watcher.get_mut(&req.root)?.remove(&req.to_string())?;
        broadcast.update_statusline(StatuslineState::Clear);
    }

    Ok(())
}

#[async_trait]
impl Watchable for BuildRequest {
    async fn trigger(
        &self,
        state: &MutexGuard<State>,
        _event: &Event,
        broadcast: &Arc<Broadcast>,
    ) -> Result<()> {
        broadcast.update_statusline(StatuslineState::Processing);
        let is_once = self.ops.is_once();
        let config = &self.settings;
        let root = &self.root;
        let target = &self.settings.target;
        let project = state.projects.get(root)?;

        if is_once {
            broadcast.info(format!("[{target}] Building ⚙"));
        }
        let (args, mut recv) = project.build(&config, None, broadcast)?;

        if !recv.recv().await.unwrap_or_default() {
            let verb = if is_once { "building" } else { "Rebuilding" };
            broadcast.error(format!("[{target}] {verb} Failed, checkout logs"));
            broadcast.log_error(format!(
                "[{target}] build args `xcodebuild {}`",
                args.join(" ")
            ));
            broadcast.update_statusline(StatuslineState::Failure);
            broadcast.open_logger();
        } else {
            broadcast.info(format!("[{target}] Built "));
            broadcast.log_info(format!("[{target}] Built Successfully "));
            if is_once {
                broadcast.update_statusline(StatuslineState::Success);
            } else {
                broadcast.update_statusline(StatuslineState::Watching);
            }
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
