use crate::broadcast::Broadcast;
use crate::project::ProjectImplementer;
use crate::store::TryGetDaemonObject;
use crate::watch::{Event, WatchService, Watchable};
use crate::Result;
use async_trait::async_trait;
use std::sync::{Arc, Weak};
use tokio::sync::{Mutex, OwnedMutexGuard};
use xbase_proto::{BuildRequest, StatuslineState};

/// Handle build Request
pub async fn handle(req: BuildRequest) -> Result<()> {
    let broadcast = req.root.try_get_broadcast().await?;
    let target = &req.settings.target;
    // let args = &req.settings.to_string();

    log::trace!("{:#?}", req);

    if req.ops.is_once() {
        let mut project = req.root.try_get_project().await?;

        req.trigger(&mut project, &Event::default(), &broadcast, Weak::new())
            .await?;
        return Ok(());
    }

    let mut watcher = req.root.try_get_watcher().await?;

    if req.ops.is_watch() {
        broadcast.success(format!("[{target}] Watching "));
        broadcast.update_statusline(StatuslineState::Watching);
        watcher.add(req)?;
    } else {
        broadcast.info(format!("[{}] Wathcer Stopped", &req.settings.target));
        watcher.remove(&req.to_string())?;
        broadcast.update_statusline(StatuslineState::Clear);
    }

    Ok(())
}

#[async_trait]
impl Watchable for BuildRequest {
    async fn trigger(
        &self,
        project: &mut OwnedMutexGuard<ProjectImplementer>,
        _event: &Event,
        broadcast: &Arc<Broadcast>,
        _watcher: Weak<Mutex<WatchService>>,
    ) -> Result<()> {
        broadcast.update_statusline(StatuslineState::Processing);
        let is_once = self.ops.is_once();
        let config = &self.settings;
        let target = &self.settings.target;

        if is_once {
            broadcast.info(format!("[{target}] Building ⚙"));
        }
        let (args, mut recv) = project.build(&config, None, broadcast)?;

        if !recv.recv().await.unwrap_or_default() {
            let verb = if is_once { "building" } else { "Rebuilding" };
            broadcast.error(format!("[{target}] {verb} Failed "));
            broadcast.log_error(format!(
                "[{target}] build args `xcodebuild {}`",
                args.join(" ")
            ));
            broadcast.update_statusline(StatuslineState::Failure);
            broadcast.open_logger();
        } else {
            broadcast.success(format!("[{target}] Built "));
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
    async fn should_trigger(&self, event: &Event) -> bool {
        event.is_content_update_event()
            || event.is_rename_event()
            || event.is_create_event()
            || event.is_remove_event()
            || !(event.path().exists() || event.is_seen())
    }

    /// A function that controls whether a watchable should be droped
    async fn should_discard(&self, _event: &Event) -> bool {
        false
    }

    /// Drop watchable for watching a given file system
    async fn discard(&self) -> Result<()> {
        Ok(())
    }
}
