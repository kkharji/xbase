use super::*;
use crate::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;
use std::sync::{Arc, Weak};
use tokio::sync::{Mutex, OwnedMutexGuard};

/// Request to build a particular project
#[derive(Debug, Serialize, Deserialize)]
pub struct BuildRequest {
    pub root: PathBuf,
    pub settings: BuildSettings,
    pub operation: Operation,
}

impl fmt::Display for BuildRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:Build:{}", self.root.display(), self.settings)
    }
}

#[async_trait]
impl RequestHandler<()> for BuildRequest {
    async fn handle(self) -> Result<()> {
        let broadcast = self.root.try_get_broadcast().await?;
        let target = &self.settings.target;
        // let args = &self.settings.to_string();

        tracing::trace!("{:#?}", self);

        if self.operation.is_once() {
            let mut project = self.root.try_get_project().await?;

            self.trigger(&mut project, &Event::default(), &broadcast, Weak::new())
                .await?;
            return Ok(());
        }

        let mut watcher = self.root.try_get_watcher().await?;

        if self.operation.is_watch() {
            broadcast.success(format!("[{target}] Watching "));
            broadcast.update_statusline(StatuslineState::Watching);
            watcher.add(self)?;
        } else {
            broadcast.info(format!("[{}] Wathcer Stopped", &self.settings.target));
            watcher.remove(&self.to_string())?;
            broadcast.update_statusline(StatuslineState::Clear);
        }

        Ok(())
    }
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
        let is_once = self.operation.is_once();
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
