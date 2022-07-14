use super::*;
use crate::*;
use process_stream::Process;
use std::path::PathBuf;
use tap::Pipe;
use tokio::process::Command;
use xclog::XCBuildSettings;

/// Simulator Device runner
pub struct SimulatorRunner {
    pub device: Device,
    pub app_id: String,
    pub output_dir: PathBuf,
}

#[async_trait::async_trait]
impl Runner for SimulatorRunner {
    async fn run<'a>(&self, task: &Task) -> Result<Process> {
        self.boot(task).await?;
        self.install(task).await?;
        let process = self.launch(task).await;
        process
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

    pub async fn boot<'a>(&self, task: &Task) -> Result<()> {
        match pid::get_pid_by_name("Simulator") {
            Err(Error::Lookup(_, _)) => {
                task.info(format!("[Simulator] Launching"));
                Command::new("open")
                    .args(&["-a", "Simulator"])
                    .spawn()?
                    .wait()
                    .await?;
                task.info("[Simulator] Connected");
            }
            Err(err) => {
                task.error(err.to_string());
            }
            _ => {}
        }

        task.info(self.booting_msg());
        if let Err(e) = self.device.boot() {
            let err: Error = e.into();
            let err_msg = err.to_string();
            if !err_msg.contains("current state Booted") {
                // task.log_error(err_msg);
            }
        }
        Ok(())
    }

    pub async fn install<'a>(&self, task: &Task) -> Result<()> {
        task.info(self.installing_msg());
        self.device
            .install(&self.output_dir)
            .pipe(|res| self.ok_or_abort(res, task))
            .await?;
        Ok(())
    }

    pub async fn launch<'a>(&self, task: &Task) -> Result<Process> {
        task.info(self.launching_msg());
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

        task.info(self.connected_msg());

        Ok(process)
    }

    async fn ok_or_abort<'a, T>(&self, res: simctl::Result<T>, task: &Task) -> Result<()> {
        if let Err(e) = res {
            let error: Error = e.into();
            task.error(error.to_string());
            Err(error)
        } else {
            Ok(())
        }
    }

    fn booting_msg(&self) -> String {
        format!("[{}] Booting", self.device.name)
    }

    fn installing_msg(&self) -> String {
        format!("[{}] Installing {}", self.device.name, self.app_id)
    }

    fn launching_msg(&self) -> String {
        format!("[{}] Launching {}", self.device.name, self.app_id)
    }

    fn connected_msg(&self) -> String {
        format!("[{}]", self.device.name)
    }
}

#[derive(Debug, serde::Serialize, derive_deref_rs::Deref)]
pub struct Devices(std::collections::HashMap<String, Device>);

impl Default for Devices {
    fn default() -> Self {
        Devices(
            simctl::Simctl::new()
                .list()
                .unwrap()
                .devices()
                .to_vec()
                .into_iter()
                .filter(|d| d.is_available)
                .map(|d| (d.udid.clone(), Device::from(d)))
                .collect(),
        )
    }
}

impl Devices {
    /// Get Device from Device lookup
    pub fn from_lookup(&self, lookup: Option<DeviceLookup>) -> Option<Device> {
        lookup.and_then(|d| self.get(&d.id)).cloned()
    }
}
