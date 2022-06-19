use crate::{
    state::State,
    watch::{Event, Watchable},
    Error, Result,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::MutexGuard;
use xbase_proto::{Client, RunRequest};
use xclog::{XCBuildSettings, XCLogger};
use {super::handler::RunServiceHandler, super::medium::RunMedium};

/// Run Service
pub struct RunService {
    pub key: String,
    pub client: Client,
    pub handler: Arc<Mutex<RunServiceHandler>>,
    pub medium: RunMedium,
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
        let build_args = state
            .projects
            .get(root)?
            .build_args(&req.settings, &device)?;
        let nvim = state.clients.get(&req.client.pid)?;

        let ref mut logger = nvim.logger();
        if !req.ops.is_watch() {
            logger.open_win().await?;
            logger.set_running(false).await?;
        }

        logger.set_title(format!("Build:{target}"));
        logger.set_direction(&req.direction);

        let build_settings = XCBuildSettings::new(root, &build_args).await?;
        let xclogger = XCLogger::new(&root, build_args)?;
        let success = logger.consume_build_logs(xclogger, false, false).await?;

        if !success {
            let msg = format!("Failed: {}", req.settings);
            nvim.echo_err(&msg).await?;
            return Err(Error::Build(msg));
        }

        logger.set_title(format!("Run:{target}"));

        let medium = RunMedium::from_device_or_settings(device, build_settings, req.settings)?;
        let process = medium.run(logger).await?;
        let handler = RunServiceHandler::new(target, req.client.clone(), process, key.clone())?;

        logger.set_running(true).await?;

        Ok(Self {
            client: req.client,
            handler: Arc::new(Mutex::new(handler)),
            medium,
            key,
        })
    }
}

#[async_trait::async_trait]
impl Watchable for RunService {
    async fn trigger(&self, state: &MutexGuard<State>, _event: &Event) -> Result<()> {
        log::info!("Running {}", self.client.abbrev_root());

        let (root, config) = (&self.client.root, &self.medium.settings());
        let mut handler = self.handler.clone().lock_owned().await;
        let mut args = state.projects.get(root)?.build_args(&config, &None)?;

        if let RunMedium::Simulator(ref sim) = self.medium {
            args.extend(sim.special_build_args())
        }

        handler.process().kill().await;
        handler.inner().abort();

        let nvim = state.clients.get(&self.client.pid)?;
        let ref mut logger = nvim.logger();

        logger.set_title(format!("Run:{}", config.target));
        let xclogger = XCLogger::new(&root, args)?;
        let success = logger.consume_build_logs(xclogger, false, false).await?;

        if !success {
            let ref msg = format!("Failed: {} ", config.to_string());
            nvim.echo_err(msg).await?;
        };

        let process = self.medium.run(logger).await?;
        *handler = RunServiceHandler::new(
            config.target.clone(),
            self.client.clone(),
            process,
            self.key.clone(),
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
