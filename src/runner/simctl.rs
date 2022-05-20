use crate::{nvim::Logger, types::SimDevice, Error, Result};
use std::path::PathBuf;
use tap::Pipe;

/// SimDevice ruuner
pub struct SimDeviceRunner {
    device: SimDevice,
    target: String,
    app_id: String,
    path_to_app: PathBuf,
    stdout_path: String,
    stderr_path: String,
}

impl SimDeviceRunner {
    pub async fn boot<'a>(&self, logger: &mut Logger<'a>) -> Result<()> {
        logger.log(self.booting_msg()).await?;
        if let Err(e) = self.device.boot() {
            let err: Error = e.into();
            let err_msg = err.to_string();
            if !err_msg.contains("current state Booted") {
                logger.log(err_msg).await?;
                logger.set_status_end(false, true).await?;
                return Err(err);
            }
        }
        Ok(())
    }

    pub async fn install<'a>(&self, logger: &mut Logger<'a>) -> Result<()> {
        logger.log(self.installing_msg()).await?;
        self.device
            .install(&self.path_to_app)
            .pipe(|res| self.ok_or_abort(res, logger))
            .await?;
        Ok(())
    }

    pub async fn launch<'a>(&self, logger: &mut Logger<'a>) -> Result<()> {
        logger.log(self.launching_msg()).await?;
        self.device
            .launch(&self.app_id)
            .stdout(&self.stdout_path)
            .stderr(&self.stderr_path)
            .exec()
            .pipe(|res| self.ok_or_abort(res, logger))
            .await?;
        logger.log(self.connected_msg()).await?;
        Ok(())
    }
    async fn ok_or_abort<'a, T>(
        &self,
        res: simctl::Result<T>,
        logger: &mut Logger<'a>,
    ) -> Result<()> {
        if let Err(e) = res {
            let error: Error = e.into();
            logger.log(error.to_string()).await?;
            logger.set_status_end(false, true).await?;
            Err(error)
        } else {
            Ok(())
        }
    }
}

impl SimDeviceRunner {
    fn booting_msg(&self) -> String {
        format!("[Run:{}] Booting {}", self.target, self.device.name)
    }

    fn installing_msg(&self) -> String {
        format!("[Run:{}] Installing {}", self.target, self.app_id)
    }

    fn launching_msg(&self) -> String {
        format!("[Run:{}] Launching {}", self.target, self.app_id)
    }

    fn connected_msg(&self) -> String {
        format!("[Run:{}] Connected", self.target)
    }
}

impl SimDeviceRunner {
    pub fn new(device: SimDevice, target: String, app_id: String, path_to_app: PathBuf) -> Self {
        let out_path = |out| format!("/tmp/{}_{out}_{}_runner.log", target, &device.udid).into();
        let stdout_path = out_path("stdout");
        let stderr_path = out_path("stderr");

        tracing::debug!(
            "SimDeviceRunner: {}: {app_id} [{path_to_app:?}]",
            device.name
        );
        Self {
            device,
            target,
            app_id,
            path_to_app,
            stdout_path,
            stderr_path,
        }
    }
}
