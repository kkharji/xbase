#![allow(dead_code)]
use crate::{Error, Result};
use process_stream::Process;
use std::path::PathBuf;
use xbase_proto::BuildSettings;
use xclog::XCBuildSettings;

pub struct Bin {
    path: PathBuf,
    info: XCBuildSettings,
    settings: BuildSettings,
}

impl Bin {
    pub fn new(info: XCBuildSettings, settings: BuildSettings) -> Self {
        Self {
            path: info.path_to_output_binary().unwrap_or_default(),
            info,
            settings,
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
    pub fn config(&self) -> &BuildSettings {
        &self.settings
    }
}
