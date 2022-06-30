use super::handler::RunServiceHandler;
use crate::broadcast::Broadcast;
use crate::run::get_runner;
use crate::{
    constants::State,
    device::Device,
    watch::{Event, Watchable},
    Result,
};
use std::path::PathBuf;
use std::sync::Arc;
use tap::Pipe;
use tokio::sync::Mutex;
use tokio::sync::MutexGuard;
use xbase_proto::{BuildSettings, RunRequest};

/// Run Service
pub struct RunService {
    pub key: String,
    pub root: PathBuf,
    pub handler: Arc<Mutex<RunServiceHandler>>,
    pub settings: BuildSettings,
    pub device: Option<Device>,
}

impl std::fmt::Display for RunService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.key)
    }
}

impl RunService {
    pub async fn new(
        state: &mut MutexGuard<'_, State>,
        req: RunRequest,
        logger: &Arc<Broadcast>,
    ) -> Result<Self> {
        let weak_logger = Arc::downgrade(&logger);
        let key = req.to_string();
        let RunRequest {
            root,
            settings,
            device,
            ..
        } = req;
        let target = &settings.target;
        let device = state.devices.from_lookup(device);
        let is_once = req.ops.is_once();

        let process =
            get_runner(state, &root, &settings, device.as_ref(), is_once, &logger).await?;

        let handler = RunServiceHandler::new(&key, target, &root, process, weak_logger)?
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

#[async_trait::async_trait]
impl Watchable for RunService {
    async fn trigger(
        &self,
        state: &MutexGuard<State>,
        _event: &Event,
        logger: &Arc<Broadcast>,
    ) -> Result<()> {
        let Self {
            key,
            root,
            settings,
            ..
        } = self;

        let mut handler = self.handler.clone().lock_owned().await;

        handler.process().abort();
        handler.inner().abort();

        let target = &settings.target;
        let device = self.device.as_ref();

        *handler = RunServiceHandler::new(
            key,
            target,
            root,
            get_runner(state, root, settings, device, false, &logger).await?,
            Arc::downgrade(logger),
        )?;

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
        let handler = self.handler.clone().lock_owned().await;
        handler.process().abort();
        handler.inner().abort();
        Ok(())
    }
}
