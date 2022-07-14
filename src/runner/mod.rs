mod device;
mod handler;
mod service;
mod simulator;

use crate::*;
use async_trait::async_trait;
use process_stream::Process;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::OwnedMutexGuard;
use xclog::XCBuildSettings;

pub use service::RunService;
pub use {device::*, service::*, simulator::*};

#[async_trait]
pub trait Runner {
    async fn run<'a>(&self, task: &Task) -> Result<Process>;
}

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
    async fn run<'a>(&self, _task: &Task) -> Result<Process> {
        if !self.path.exists() {
            return Err(Error::Run(format!("{:?} doesn't exist!", self.path)));
        }

        Ok(Process::new(&self.path))
    }
}

pub async fn get_runner<'a>(
    project: &mut OwnedMutexGuard<ProjectImplementer>,
    settings: &BuildSettings,
    device: Option<&Device>,
    _is_once: bool,
    broadcast: &Arc<Broadcast>,
) -> Result<Process> {
    let target = &settings.target;
    let (runner, _args, mut recv) = project.get_runner(&settings, device, broadcast)?;

    if !recv.recv().await.unwrap_or_default() {
        return Err(crate::Error::Run(format!("{target} build failed")));
    }
    let task = Task::new(TaskKind::Run, target, broadcast.clone());

    let process = runner.run(&task).await?;
    Ok(process)
}
