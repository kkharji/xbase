use super::handler::RunServiceHandler;
use crate::run::get_runner;
use crate::{
    device::Device,
    state::State,
    watch::{Event, Watchable},
    Result,
};
use process_stream::ProcessExt;
use std::sync::Arc;
use tap::Pipe;
use tokio::sync::Mutex;
use tokio::sync::MutexGuard;
use xbase_proto::{BuildSettings, Client, RunRequest};

/// Run Service
pub struct RunService {
    pub key: String,
    pub client: Client,
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
    pub async fn new(state: &mut MutexGuard<'_, State>, req: RunRequest) -> Result<Self> {
        let key = req.to_string();
        let RunRequest {
            client,
            settings,
            device,
            ..
        } = req;
        let target = &settings.target;
        let device = state.devices.from_lookup(device);
        let is_once = req.ops.is_once();

        let process = get_runner(state, &client, &settings, device.as_ref(), is_once).await?;
        let handler = RunServiceHandler::new(&key, target, &client, process)?
            .pipe(Mutex::new)
            .pipe(Arc::new);

        Ok(Self {
            device,
            handler,
            client,
            settings,
            key,
        })
    }
}

#[async_trait::async_trait]
impl Watchable for RunService {
    async fn trigger(&self, state: &MutexGuard<State>, _event: &Event) -> Result<()> {
        let Self {
            key,
            client,
            settings,
            ..
        } = self;

        let mut handler = self.handler.clone().lock_owned().await;

        handler.process().kill().await;
        handler.inner().abort();

        let target = &settings.target;
        let device = self.device.as_ref();

        *handler = RunServiceHandler::new(
            key,
            target,
            client,
            get_runner(state, client, settings, device, false).await?,
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
        handler.process().kill().await;
        handler.inner().abort();
        Ok(())
    }
}
