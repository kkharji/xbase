use super::handler::RunServiceHandler;
use crate::{
    device::Device,
    state::State,
    watch::{Event, Watchable},
    Error, Result,
};
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
        let target = req.settings.target.clone();
        let ref root = req.client.root;
        let device = state.devices.from_lookup(req.device);
        let nvim = state.clients.get(&req.client.pid)?;
        let logger = &mut nvim.logger();

        logger.set_direction(&req.direction);

        if !req.ops.is_watch() {
            logger.open_win().await?;
            logger.set_running(false).await?;
        }

        let (runner, stream) = state
            .projects
            .get(root)?
            .get_runner(&req.settings, device.as_ref())?;

        logger.set_title(format!("Build:{target}"));
        let success = logger.consume_build_logs(stream, false, false).await?;
        if !success {
            let msg = format!("Build failed {}", &req.settings);
            logger.nvim.echo_err(&msg).await?;
            return Err(Error::Build(msg));
        }
        logger.set_title(format!("Run:{target}"));
        logger.set_running(true).await?;

        let process = runner.run(logger).await?;
        let handler = RunServiceHandler::new(&key, &target, &req.client, process)?
            .pipe(Mutex::new)
            .pipe(Arc::new);

        Ok(Self {
            device,
            handler,
            client: req.client,
            settings: req.settings,
            key,
        })
    }
}

#[async_trait::async_trait]
impl Watchable for RunService {
    async fn trigger(&self, state: &MutexGuard<State>, _event: &Event) -> Result<()> {
        let (root, config, pid) = (&self.client.root, &self.settings, &self.client.pid);
        let mut handler = self.handler.clone().lock_owned().await;

        handler.process().kill().await;
        handler.inner().abort();

        let nvim = state.clients.get(pid)?;
        let logger = &mut nvim.logger();

        let (runner, stream) = state
            .projects
            .get(root)?
            .get_runner(config, self.device.as_ref())?;

        logger.set_title(format!("Build:{}", self.settings.target));
        let success = logger.consume_build_logs(stream, false, false).await?;
        if !success {
            let msg = format!("Build failed {}", &self.settings);
            logger.nvim.echo_err(&msg).await?;
            return Err(Error::Build(msg));
        }

        *handler = RunServiceHandler::new(
            &self.key,
            &config.target,
            &self.client,
            runner.run(logger).await?,
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
