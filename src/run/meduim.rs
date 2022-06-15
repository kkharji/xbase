#![allow(dead_code)]

use super::{bin::Bin, simulator::Simulator};
use crate::nvim::Logger;
use crate::types::{BuildConfiguration, Device};
use crate::Result;
use process_stream::Process;
use tap::Pipe;
use xclog::XCBuildSettings;

/// Runner to run the built binary
pub enum RunMedium {
    Simulator(Simulator),
    Bin(Bin),
}

impl RunMedium {
    pub fn from_device_or_settings(
        device: Option<Device>,
        settings: XCBuildSettings,
        config: BuildConfiguration,
    ) -> Result<Self> {
        match device {
            Some(device) => Self::Simulator(Simulator::new(device, settings, config)),
            None => Self::Bin(Bin::new(settings, config)),
        }
        .pipe(Ok)
    }

    pub async fn run<'a>(&self, logger: &mut Logger<'a>) -> Result<Process> {
        match self {
            RunMedium::Simulator(simulator) => {
                simulator.boot(logger).await?;
                simulator.install(logger).await?;
                simulator.launch(logger).await
            }
            RunMedium::Bin(bin) => bin.launch().await,
        }
    }

    pub fn config(&self) -> &BuildConfiguration {
        match self {
            RunMedium::Simulator(s) => s.config(),
            RunMedium::Bin(b) => b.config(),
        }
    }

    pub fn info(&self) -> &XCBuildSettings {
        match self {
            RunMedium::Simulator(s) => s.info(),
            RunMedium::Bin(b) => b.info(),
        }
    }

    pub fn target(&self) -> &str {
        match self {
            RunMedium::Simulator(s) => s.config().target.as_str(),
            RunMedium::Bin(b) => b.config().target.as_str(),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            RunMedium::Simulator(s) => s.name.as_ref(),
            RunMedium::Bin(_) => "Bin",
        }
    }
}
