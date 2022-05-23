mod bin;
mod handler;
mod meduim;
mod simulator;

use crate::{
    client::Client, daemon::RunRequest, state::State, types::Device, xcode::build_with_logger,
    Error, Result,
};
use tokio::sync::MutexGuard;
use xcodebuild::runner::build_settings;
use {handler::RunServiceHandler, meduim::RunMedium};

/// Run Service
pub struct RunService {
    pub client: Client,
    pub handler: RunServiceHandler,
    pub medium: RunMedium,
}

impl RunService {
    pub async fn new(state: &mut MutexGuard<'_, State>, req: RunRequest) -> Result<Self> {
        let ref target = req.config.target;
        let ref root = req.client.root;
        let device = state.devices.from_lookup(req.device);
        let build_args = req.config.args(root, &device)?;
        let nvim = req.client.nvim(state)?;

        let ref mut logger = nvim.logger();

        logger.set_title(format!("Run:{target}"));
        logger.open_win().await?;
        logger.set_direction(&req.direction);
        logger.set_running().await?;

        let build_settings = build_settings(root, &build_args).await?;
        let build_success = build_with_logger(logger, root, &build_args, false, false).await?;

        if !build_success {
            let msg = format!("Failed: {}", req.config);
            nvim.echo_err(&msg).await?;
            return Err(Error::Build(msg));
        }

        let medium = RunMedium::from_device_or_settings(device, build_settings, req.config)?;
        let process = medium.run(logger).await?;
        let handler = RunServiceHandler::new(req.client.clone(), process)?;

        Ok(Self {
            client: req.client,
            handler,
            medium,
        })
    }
}
