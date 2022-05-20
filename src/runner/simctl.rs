use crate::{types::SimDevice, Error};

use super::*;

impl Runner {
    pub async fn run_with_simctl(self, settings: BuildSettings) -> Result<JoinHandle<Result<()>>> {
        let nvim = self.state.clients.get(&self.client.pid)?;
        let mut logger = nvim.new_logger("Run", &self.target, &self.direction);

        let app_id = settings.product_bundle_identifier;
        let path_to_app = settings.metal_library_output_dir;

        tracing::debug!("{app_id}: {:?}", path_to_app);

        logger.log_title().await?;
        logger.open_win().await?;

        let mut device = get_device(&self.state, self.udid)?;

        // NOTE: This is required so when neovim exist this should also exit
        let state = DAEMON_STATE.clone().lock_owned().await;

        tokio::spawn(async move {
            let nvim = state.clients.get(&self.client.pid)?;
            let ref mut logger = nvim.new_logger("Run", &self.target, &self.direction);

            logger.set_running().await?;

            device.try_boot(logger).await?;
            device.try_install(&path_to_app, &app_id, logger).await?;
            device.try_launch(&app_id, logger).await?;

            let mut state = DAEMON_STATE.clone().lock_owned().await;

            // TODO(simctl): device might change outside state
            state.devices.insert(device);

            // TODO: Remove and replace with app logs
            logger.set_status_end(true, false).await?;

            state.sync_client_state().await?;

            Ok(())
        })
        .pipe(Ok)
    }
}

fn get_device<'a>(state: &'a OwnedMutexGuard<State>, udid: Option<String>) -> Result<SimDevice> {
    if let Some(udid) = udid {
        state.devices.iter().find(|d| d.udid == udid).cloned()
    } else {
        None
    }
    .ok_or_else(|| Error::Run("udid not found!!".to_string()))
}
