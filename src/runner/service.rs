use super::{handler::RunServiceHandler, *};
use crate::*;
use std::path::PathBuf;
use std::sync::{Arc, Weak};
use tap::Pipe;
use tokio::sync::{Mutex, OwnedMutexGuard};

/// Run Service
pub struct RunService {
    pub key: String,
    pub root: PathBuf,
    pub handler: Arc<Mutex<RunServiceHandler>>,
    pub settings: BuildSettings,
    pub device: Option<Device>,
}

impl RunService {
    pub async fn new(
        key: String,
        root: PathBuf,
        settings: BuildSettings,
        device: DeviceLookup,
        operation: Operation,
        broadcast: &Arc<Broadcast>,
        watcher: Weak<Mutex<WatchService>>,
        project: &mut OwnedMutexGuard<ProjectImplementer>,
    ) -> Result<Self> {
        let weak_logger = Arc::downgrade(&broadcast);
        let target = &settings.target;
        let device = devices().from_lookup(device);
        let is_once = operation.is_once();

        let process = get_runner(project, &settings, device.as_ref(), is_once, &broadcast).await?;

        let handler = RunServiceHandler::new(&key, target, process, weak_logger, watcher)?
            .pipe(Mutex::new)
            .pipe(Arc::new);

        Ok(Self {
            device,
            handler,
            root,
            settings,
            key,
        })
    }
}

impl std::fmt::Display for RunService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.key)
    }
}

#[async_trait::async_trait]
impl Watchable for RunService {
    async fn trigger(
        &self,
        project: &mut OwnedMutexGuard<ProjectImplementer>,
        _event: &Event,
        broadcast: &Arc<Broadcast>,
        watcher: Weak<Mutex<WatchService>>,
    ) -> Result<()> {
        let Self { key, settings, .. } = self;

        let mut handler = self.handler.clone().lock_owned().await;

        handler.process().abort();
        handler.inner().abort();

        let target = &settings.target;
        let device = self.device.as_ref();

        *handler = RunServiceHandler::new(
            key,
            target,
            get_runner(project, settings, device, false, &broadcast).await?,
            Arc::downgrade(broadcast),
            watcher,
        )?;

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
        let handler = self.handler.clone().lock_owned().await;
        handler.process().abort();
        handler.inner().abort();
        Ok(())
    }
}
