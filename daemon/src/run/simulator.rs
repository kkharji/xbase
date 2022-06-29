use crate::broadcast::{self, Broadcast};
use crate::device::Device;
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
    async fn run<'a>(&self, broadcast: &Broadcast) -> Result<Process> {
        self.boot(broadcast).await?;
        self.install(broadcast).await?;
        self.launch(broadcast).await
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

    pub async fn boot<'a>(&self, broadcast: &Broadcast) -> Result<()> {
        match pid::get_by_name("Simulator") {
            Err(Error::Lookup(_, _)) => {
                let msg = format!("[Simulator] Launching");
                log::info!("{msg}");
                broadcast::log_info!(broadcast, "{msg}")?;
                tokio::process::Command::new("open")
                    .args(&["-a", "Simulator"])
                    .spawn()?
                    .wait()
                    .await?;
                broadcast::log_info!(broadcast, "[Simulator] Connected")?;
                broadcast::log_info!(broadcast, "{}", fmt::separator())?;
            }
            Err(err) => {
                let msg = err.to_string();
                log::error!("{msg}");
                broadcast::log_error!(broadcast, "{}", msg)?;
            }
            _ => {}
        }

        broadcast::log_info!(broadcast, "{}", self.booting_msg())?;
        if let Err(e) = self.device.boot() {
            let err: Error = e.into();
            let err_msg = err.to_string();
            if !err_msg.contains("current state Booted") {
                broadcast::log_error!(broadcast, "err_msg")?;
            }
        }
        Ok(())
    }

    pub async fn install<'a>(&self, broadcast: &Broadcast) -> Result<()> {
        broadcast::log_info!(broadcast, "{}", self.installing_msg())?;
        self.device
            .install(&self.output_dir)
            .pipe(|res| self.ok_or_abort(res, broadcast))
            .await?;
        Ok(())
    }

    pub async fn launch<'a>(&self, broadcast: &Broadcast) -> Result<Process> {
        broadcast::log_info!(broadcast, "{}", self.launching_msg())?;
        let mut process = Process::new("xcrun");
        let args = &[
            "simctl",
            "launch",
            "--terminate-running-process",
            "--console-pty",
            &self.device.udid,
            &self.app_id,
        ];

        process.args(args);

        broadcast::log_info!(broadcast, "{}", self.connected_msg())?;
        broadcast::log_info!(broadcast, "{}", fmt::separator())?;

        Ok(process)
    }

    async fn ok_or_abort<'a, T>(
        &self,
        res: simctl::Result<T>,
        broadcast: &Broadcast,
    ) -> Result<()> {
        if let Err(e) = res {
            let error: Error = e.into();
            broadcast::log_error!(broadcast, "{}", error.to_string())?;
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
