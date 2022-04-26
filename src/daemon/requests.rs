use anyhow::Result;

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

/// Requirement for daemon actions
#[async_trait::async_trait]
#[cfg(feature = "daemon")]
pub trait DaemonRequestHandler {
    async fn handle(&self, state: super::DaemonState) -> Result<()>;
}

/// Representations of all the supported daemon requests
#[derive(Debug)]
pub enum DaemonRequest {
    Build(Build),
    Run(Run),
    RenameFile(RenameFile),
    Register(Register),
    Drop(Drop),
}

impl DaemonRequest {
    #[cfg(feature = "daemon")]
    /// Handle daemon request
    pub async fn handle(&self, state: super::DaemonState) -> Result<()> {
        use DaemonRequest::*;

        match self {
            Build(c) => c.handle(state).await,
            Run(c) => c.handle(state).await,
            RenameFile(c) => c.handle(state).await,
            Register(c) => c.handle(state).await,
            Drop(c) => c.handle(state).await,
        }
    }

    /// Parse [`super::Daemon`] request from string
    pub fn parse(str: &str) -> Result<Self> {
        let mut args = str.split(" ").collect::<Vec<&str>>();
        Ok(match args.remove(0) {
            Build::KEY => Self::Build(args.try_into()?),
            Run::KEY => Self::Run(args.try_into()?),
            RenameFile::KEY => Self::RenameFile(args.try_into()?),
            Register::KEY => Self::Register(args.try_into()?),
            Drop::KEY => Self::Drop(args.try_into()?),
            cmd => anyhow::bail!("Unknown command messsage: {cmd}"),
        })
    }
}
