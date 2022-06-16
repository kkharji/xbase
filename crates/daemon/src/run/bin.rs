#![allow(dead_code)]
use crate::types::BuildConfiguration;
use crate::{Error, Result};
use process_stream::Process;
use std::path::PathBuf;
use xclog::XCBuildSettings;

pub struct Bin {
    path: PathBuf,
    info: XCBuildSettings,
    config: BuildConfiguration,
}

impl Bin {
    pub fn new(info: XCBuildSettings, config: BuildConfiguration) -> Self {
        Self {
            path: info.path_to_output_binary().unwrap_or_default(),
            info,
            config,
        }
    }

    pub async fn launch(&self) -> Result<Process> {
        if !self.path.exists() {
            return Err(Error::Run(format!("{:?} doesn't exist!", self.path)));
        }

        Ok(Process::new(&self.path))
    }

    /// Get a reference to the bin's info.
    #[must_use]
    pub fn info(&self) -> &XCBuildSettings {
        &self.info
    }

    /// Get a reference to the bin's config.
    #[must_use]
    pub fn config(&self) -> &BuildConfiguration {
        &self.config
    }
}
