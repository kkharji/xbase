use crate::device::Device;
use crate::nvim::Logger;
use crate::run::Runner;
use crate::util::{fmt, pid};
use crate::{Error, Result};
use process_stream::Process;
use std::path::PathBuf;
use tap::Pipe;
use xclog::XCBuildSettings;

/// Simulator Device runner
pub struct SimulatorRunner {
    pub device: Device,
    pub app_id: String,
    pub output_dir: PathBuf,
}

#[async_trait::async_trait]
impl Runner for SimulatorRunner {
    async fn run<'a>(&self, logger: &mut Logger<'a>) -> Result<Process> {
        self.boot(logger).await?;
        self.install(logger).await?;
        self.launch(logger).await
    }
}

impl SimulatorRunner {
    pub fn new(device: Device, info: &XCBuildSettings) -> Self {
        Self {
            device,
            app_id: info.product_bundle_identifier.clone(),
            output_dir: info.metal_library_output_dir.clone(),
        }
    }

    pub async fn boot<'a>(&self, logger: &mut Logger<'a>) -> Result<()> {
        match pid::get_by_name("Simulator") {
            Err(Error::NotFound(_, _)) => {
                let msg = format!("[Simulator] Launching");
                log::info!("{msg}");
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
                log::error!("{msg}");
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
            .install(&self.output_dir)
            .pipe(|res| self.ok_or_abort(res, logger))
            .await?;
        Ok(())
    }

    pub async fn launch<'a>(&self, logger: &mut Logger<'a>) -> Result<Process> {
        logger.append(self.launching_msg()).await?;
        let mut process = Process::new("xcrun");
        let args = &[
            "simctl",
            "launch",
            "--terminate-running-process",
            "--console-pty",
            &self.device.udid,
            &self.app_id,
        ];

        log::debug!("Launching app with {args:?}");

        process.args(args);

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

    fn booting_msg(&self) -> String {
        format!("Booting {}", self.device.name)
    }

    fn installing_msg(&self) -> String {
        format!("Installing {}", self.app_id)
    }

    fn launching_msg(&self) -> String {
        format!("Launching {}", self.app_id)
    }

    fn connected_msg(&self) -> String {
        format!("Connected")
    }
}
