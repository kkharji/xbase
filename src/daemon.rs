//! Handle requests from neovim and manage dev workflow
#[cfg(feature = "daemon")]
mod nvim;
mod requests;
pub mod state;

pub use requests::*;

#[cfg(feature = "daemon")]
pub use state::DaemonState;

#[cfg(feature = "lua")]
use crate::util::mlua::LuaExtension;

#[cfg(feature = "lua")]
use mlua::prelude::*;

#[cfg(feature = "daemon")]
use anyhow::Result;

pub const DAEMON_SOCKET_PATH: &str = "/tmp/xcodebase-daemon.socket";
pub const DAEMON_BINARY: &str =
    "/Users/tami5/repos/neovim/xcodebase.nvim/target/debug/xcodebase-daemon";

/// Representation of daemon
pub struct Daemon;

/// Requirement for daemon actions
#[cfg(feature = "daemon")]
#[async_trait::async_trait]
pub trait DaemonRequestHandler<T> {
    async fn handle(&self, state: DaemonState) -> Result<()>;
    fn parse(args: Vec<&str>) -> Result<T>;
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

#[cfg(feature = "daemon")]
impl DaemonRequest {
    /// Handle daemon request
    pub async fn handle(&self, state: DaemonState) -> Result<()> {
        match self {
            DaemonRequest::Build(c) => c.handle(state).await,
            DaemonRequest::Run(c) => c.handle(state).await,
            DaemonRequest::RenameFile(c) => c.handle(state).await,
            DaemonRequest::Register(c) => c.handle(state).await,
            DaemonRequest::Drop(c) => c.handle(state).await,
        }
    }

    /// Parse [`super::Daemon`] request from string
    pub fn parse(str: &str) -> Result<Self> {
        let mut args = str.split(" ").collect::<Vec<&str>>();
        Ok(match args.remove(0) {
            Build::KEY => Self::Build(Build::parse(args)?),
            Run::KEY => Self::Run(Run::parse(args)?),
            RenameFile::KEY => Self::RenameFile(RenameFile::parse(args)?),
            Register::KEY => Self::Register(Register::parse(args)?),
            Drop::KEY => Self::Drop(Drop::parse(args)?),
            cmd => anyhow::bail!("Unknown command messsage: {cmd}"),
        })
    }
}

#[cfg(feature = "lua")]
impl Daemon {
    /// Representation of daemon table in lua
    pub fn lua(lua: &mlua::Lua) -> LuaResult<LuaTable> {
        let table = lua.create_table()?;
        table.set("is_running", lua.create_function(Self::is_running)?)?;
        table.set("ensure", lua.create_function(Self::ensure)?)?;
        table.set("register", lua.create_function(Register::lua)?)?;
        table.set("drop", lua.create_function(Drop::lua)?)?;
        Ok(table)
    }

    /// Check if Daemon is running
    pub fn is_running(_: &mlua::Lua, _: ()) -> LuaResult<bool> {
        match std::os::unix::net::UnixStream::connect(DAEMON_SOCKET_PATH) {
            Ok(stream) => Ok(stream.shutdown(std::net::Shutdown::Both).ok().is_some()),
            Err(_) => Ok(false),
        }
    }

    /// Ensure that daemon is currently running in background
    pub fn ensure(lua: &mlua::Lua, _: ()) -> LuaResult<bool> {
        if Self::is_running(lua, ()).unwrap() {
            Ok(false)
        } else if std::process::Command::new(DAEMON_BINARY).spawn().is_ok() {
            lua.info("Spawned Background Server")?;
            Ok(true)
        } else {
            panic!("Unable to spawn background server");
        }
    }

    /// Pass args to running daemon
    pub fn execute(args: &[&str]) -> LuaResult<()> {
        use std::io::Write;
        match std::os::unix::net::UnixStream::connect(DAEMON_SOCKET_PATH) {
            Ok(mut stream) => {
                stream.write_all(args.join(" ").as_str().as_ref())?;
                stream.flush().map_err(mlua::Error::external)
            }
            Err(e) => Err(mlua::Error::external(format!(
                "Fail to execute {:#?}: {e}",
                args
            ))),
        }
    }
}
