use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub message: Message,
}

impl Request {
    pub fn read(value: String) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(value.trim())?)
    }
}

/// Daemon Message
#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Build(Build),
    Run(Run),
    ProjectInfo(ProjectInfo),
    Register(Register),
    Drop(Drop),
    RenameFile(RenameFile),
    WatchStart(WatchStart),
    WatchStop(WatchStop),
}

#[cfg(feature = "daemon")]
impl Message {
    pub async fn handle(self, state: super::DaemonState) -> anyhow::Result<()> {
        match self {
            Self::Build(c) => Handler::handle(c, state).await,
            Self::Run(c) => Handler::handle(c, state).await,
            Self::RenameFile(c) => Handler::handle(c, state).await,
            Self::Register(c) => Handler::handle(c, state).await,
            Self::Drop(c) => Handler::handle(c, state).await,
            Self::ProjectInfo(c) => Handler::handle(c, state).await,
            Self::WatchStart(c) => Handler::handle(c, state).await,
            Self::WatchStop(c) => Handler::handle(c, state).await,
        }
    }
}

/// Requirement for daemon handling request
#[cfg(feature = "daemon")]
#[async_trait::async_trait]
pub trait Handler: std::fmt::Debug + Sized {
    async fn handle(self, _state: super::DaemonState) -> anyhow::Result<()> {
        tracing::error!("Not Implemented! {:#?}", self);
        Ok(())
    }
}

/// Requirement for daemon sending request
#[cfg(feature = "lua")]
pub trait Requester<'a, M: Into<Request> + std::fmt::Debug + FromLua<'a>> {
    fn request(lua: &Lua, msg: M) -> LuaResult<()> {
        Self::pre(lua, &msg)?;
        Daemon::execute(lua, msg)
    }

    fn pre(lua: &Lua, msg: &M) -> LuaResult<()> {
        lua.trace(&format!("{:?}", msg))
    }
}
