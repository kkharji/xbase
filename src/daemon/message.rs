use super::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "lua")]
use crate::util::mlua::LuaExtension;
#[cfg(feature = "lua")]
use mlua::prelude::*;

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
    Register(Register),
    Drop(Drop),
    RenameFile(RenameFile),
    WatchTarget(WatchTarget),
}

#[cfg(feature = "daemon")]
impl Message {
    pub async fn handle(self) -> anyhow::Result<()> {
        match self {
            Self::Build(c) => Handler::handle(c).await,
            Self::Run(c) => Handler::handle(c).await,
            Self::RenameFile(c) => Handler::handle(c).await,
            Self::Register(c) => Handler::handle(c).await,
            Self::Drop(c) => Handler::handle(c).await,
            Self::WatchTarget(c) => Handler::handle(c).await,
        }
    }
}

/// Requirement for daemon handling request
#[cfg(feature = "daemon")]
#[async_trait::async_trait]
pub trait Handler: std::fmt::Debug + Sized {
    async fn handle(self) -> anyhow::Result<()> {
        tracing::error!("Not Implemented! {:#?}", self);
        Ok(())
    }
}

/// Requirement for daemon sending request
#[cfg(feature = "lua")]
pub trait Requester<'a, M: Into<Request> + std::fmt::Debug + FromLua<'a>> {
    fn request(lua: &Lua, msg: M) -> LuaResult<()> {
        Self::pre(lua, &msg)?;

        use crate::constants::DAEMON_SOCKET_PATH;
        use std::io::Write;
        use std::os::unix::net::UnixStream;

        let req: Request = msg.into();
        let mut stream = UnixStream::connect(DAEMON_SOCKET_PATH)
            .map_err(|e| format!("Connect: {e} and execute: {:#?}", req))
            .to_lua_err()?;

        serde_json::to_vec(&req)
            .map(|value| stream.write_all(&value))
            .to_lua_err()??;

        stream.flush().to_lua_err()
    }

    fn pre(lua: &Lua, msg: &M) -> LuaResult<()> {
        lua.trace(&format!("{:?}", msg))
    }
}
