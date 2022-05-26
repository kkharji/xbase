use crate::{
    nvim::Logger,
    types::{BuildConfiguration, Device},
    util::{fmt, pid},
    Error, Result,
};
use process_stream::Process;
use std::path::PathBuf;
use tap::Pipe;
use xcodebuild::parser::BuildSettings;

/// Simulator Device runner
#[derive(derive_deref_rs::Deref)]
pub struct Simulator {
    #[deref]
    device: Device,
    info: BuildSettings,
    config: BuildConfiguration,
}

impl Simulator {
    pub fn new(device: Device, build_settings: BuildSettings, config: BuildConfiguration) -> Self {
        Self {
            device,
            config,
            info: build_settings,
        }
    }

    pub async fn boot<'a>(&self, logger: &mut Logger<'a>) -> Result<()> {
        match pid::get_by_name("Simulator") {
            Err(Error::NotFound(_, _)) => {
                let msg = format!("[Simulator] Launching");
                tracing::info!("{msg}");
                logger.append(msg).await?;
                tokio::process::Command::new("open")
                    .args(&["-a", "Simulator"])
                    .spawn()?
                    .wait()
                    .await?;
                let msg = format!("[Simulator] Connected");
                logger.append(msg).await?;
                logger.append(fmt::separator()).await?;
            }
            Err(err) => {
                let msg = err.to_string();
                tracing::error!("{msg}");
                logger.append(msg).await?;
            }
            _ => {}
        }

        logger.append(self.booting_msg()).await?;
        if let Err(e) = self.device.boot() {
            let err: Error = e.into();
            let err_msg = err.to_string();
            if !err_msg.contains("current state Booted") {
                logger.append(err_msg).await?;
                logger.set_status_end(false, true).await?;
                return Err(err);
            }
        }
        Ok(())
    }

    pub async fn install<'a>(&self, logger: &mut Logger<'a>) -> Result<()> {
        logger.append(self.installing_msg()).await?;
        self.device
            .install(&self.output_dir())
            .pipe(|res| self.ok_or_abort(res, logger))
            .await?;
        Ok(())
    }

    pub async fn launch<'a>(&self, logger: &mut Logger<'a>) -> Result<Process> {
        logger.append(self.launching_msg()).await?;
        let mut process = Process::new("xcrun");

        process.args(&[
            "simctl",
            "launch",
            "--terminate-running-process",
            "--console",
        ]);
        process.arg(&self.device.udid);
        process.arg(&self.info.product_bundle_identifier);
        process.kill_on_drop(true);

        logger.append(self.connected_msg()).await?;
        logger.append(fmt::separator()).await?;

        Ok(process)
    }

    async fn ok_or_abort<'a, T>(
        &self,
        res: simctl::Result<T>,
        logger: &mut Logger<'a>,
    ) -> Result<()> {
        if let Err(e) = res {
            let error: Error = e.into();
            logger.append(error.to_string()).await?;
            logger.set_status_end(false, true).await?;
            Err(error)
        } else {
            Ok(())
        }
    }

    /// Get application identifier
    pub fn app_id(&self) -> &String {
        &self.info.product_bundle_identifier
    }

    /// Get directory path to where the build output
    pub fn output_dir(&self) -> &PathBuf {
        &self.info.metal_library_output_dir
    }

    /// Get a reference to the simulator's info.
    #[must_use]
    pub fn info(&self) -> &BuildSettings {
        &self.info
    }

    /// Get a reference to the simulator's config.
    #[must_use]
    pub fn config(&self) -> &BuildConfiguration {
        &self.config
    }
}

impl Simulator {
    fn booting_msg(&self) -> String {
        format!("[{}] Booting {}", self.config.target, self.device.name)
    }

    fn installing_msg(&self) -> String {
        format!("[{}] Installing {}", self.config.target, self.app_id())
    }

    fn launching_msg(&self) -> String {
        format!("[{}] Launching {}", self.config.target, self.app_id())
    }

    fn connected_msg(&self) -> String {
        format!("[{}] Connected", self.config.target)
    }
}
