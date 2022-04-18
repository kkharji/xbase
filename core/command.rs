use crate::state::SharedState;
use anyhow::{bail, Result};
use async_trait::async_trait;

mod build;
mod drop;
mod register;
mod rename_file;
mod run;

pub use build::Build;
pub use drop::Drop;
pub use register::Register;
pub use rename_file::RenameFile;
pub use run::Run;

#[async_trait]
pub trait DaemonCommand {
    async fn handle(&self, state: SharedState) -> anyhow::Result<()>;
}

#[derive(Debug)]
pub enum Command {
    Build(Build),
    Run(Run),
    RenameFile(RenameFile),
    Register(Register),
    Drop(Drop),
}

impl Command {
    pub async fn handle(&self, state: SharedState) -> Result<()> {
        match self {
            Command::Build(c) => c.handle(state).await,
            Command::Run(c) => c.handle(state).await,
            Command::RenameFile(c) => c.handle(state).await,
            Command::Register(c) => c.handle(state).await,
            Command::Drop(c) => c.handle(state).await,
        }
    }

    pub fn parse(str: &str) -> Result<Self> {
        let mut args = str.split(" ").collect::<Vec<&str>>();
        let cmd = args.remove(0);
        match cmd {
            "build" => Ok(Self::Build(Build::new(args)?)),
            "run" => Ok(Self::Run(Run::new(args)?)),
            "rename_file" => Ok(Self::RenameFile(RenameFile::new(args)?)),
            "register" => Ok(Self::Register(Register::new(args)?)),
            "drop" => Ok(Self::Drop(Drop::new(args)?)),
            _ => bail!("Unknown command messsage: {cmd}"),
        }
    }
}
