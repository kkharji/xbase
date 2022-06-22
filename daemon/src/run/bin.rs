use crate::run::Logger;
use crate::{Error, Result};
use process_stream::Process;
use std::path::{Path, PathBuf};
use xclog::XCBuildSettings;

use super::Runner;

pub struct BinRunner {
    path: PathBuf,
}

impl BinRunner {
    pub fn from_build_info(info: &XCBuildSettings) -> Self {
        let path = info.path_to_output_binary().unwrap_or_default();
        Self { path }
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref().into();
        Self { path }
    }
}

#[async_trait::async_trait]
impl Runner for BinRunner {
    async fn run<'a>(&self, _logger: &mut Logger<'a>) -> Result<Process> {
        if !self.path.exists() {
            return Err(Error::Run(format!("{:?} doesn't exist!", self.path)));
        }

        Ok(Process::new(&self.path))
    }
}
