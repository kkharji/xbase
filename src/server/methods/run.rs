use super::*;
use crate::{runner::*, *};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::{fmt, sync::Weak};
use tap::Pipe;
use tokio::sync::{Mutex, OwnedMutexGuard};

/// Request to Run a particular project.
#[derive(Debug, Serialize, Deserialize)]
pub struct RunRequest {
    pub root: PathBuf,
    pub settings: BuildSettings,
    #[serde(deserialize_with = "util::de::value_or_default")]
    pub device: DeviceLookup,
    #[serde(deserialize_with = "util::de::value_or_default")]
    pub operation: Operation,
}

impl fmt::Display for RunRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let root = self.root.display();
        let device = if let Some(name) = self.device.name.as_ref() {
            name.to_string()
        } else {
            "Bin".into()
        };
        let settings = &self.settings;
        write!(f, "{root}:Run:{device}:{settings}",)
    }
}

#[async_trait]
impl RequestHandler<()> for RunRequest {
    async fn handle(self) -> Result<()> {
        tracing::trace!("{:#?}", self);

        let ref key = self.to_string();
        let broadcast = self.root.try_get_broadcast().await?;
        let mut project = self.root.try_get_project().await?;

        let watcher = self.root.try_get_mutex_watcher().await?;
        let weak_watcher = Arc::downgrade(&watcher);

        if self.operation.is_once() {
            // TODO(run): might want to keep track of ran services
            // RunService::new(&mut project, self, &broadcast, weak_watcher).await?;
            self.into_service(&broadcast, weak_watcher, &mut project)
                .await?;

            return Ok(Default::default());
        }

        let mut watcher = watcher.lock().await;

        if self.operation.is_watch() {
            broadcast.update_statusline(StatuslineState::Watching);
            if watcher.contains_key(key) {
                broadcast.warn(format!("Already watching with {key}!!"));
            } else {
                self.into_service(&broadcast, weak_watcher, &mut project)
                    .await?
                    .pipe(|s| watcher.add(s))?;
            }
        } else {
            let listener = watcher.remove(&self.to_string())?;
            listener.discard().await?;
            broadcast.info(format!("[{}] Watcher Stopped", &self.settings.target));
            broadcast.update_statusline(StatuslineState::Clear);
        }

        Ok(())
    }
}

impl RunRequest {
    async fn into_service(
        self,
        broadcast: &Arc<Broadcast>,
        watcher: Weak<Mutex<WatchService>>,
        project: &mut OwnedMutexGuard<ProjectImplementer>,
    ) -> Result<RunService> {
        RunService::new(
            self.to_string(),
            self.root,
            self.settings,
            self.device,
            self.operation,
            broadcast,
            watcher,
            project,
        )
        .await
    }
}
