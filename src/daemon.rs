use anyhow::{bail, Context, Result};
mod command;
pub mod state;

pub use command::*;

#[cfg(feature = "daemon")]
pub use state::DaemonState;

pub const DAEMON_SOCKET_PATH: &str = "/tmp/xcodebase-daemon.socket";
pub const DAEMON_BINARY: &str =
    "/Users/tami5/repos/neovim/xcodebase.nvim/target/debug/xcodebase-daemon";

pub struct Daemon {
    #[cfg(feature = "daemon")]
    pub state: std::sync::Arc<tokio::sync::Mutex<state::DaemonStateData>>,
    #[cfg(feature = "daemon")]
    pub listener: tokio::net::UnixListener,
}

impl Daemon {
    #[cfg(feature = "daemon")]
    pub fn new() -> Self {
        Self {
            state: Default::default(),
            listener: tokio::net::UnixListener::bind(DAEMON_SOCKET_PATH).unwrap(),
        }
    }

    /// Spawn new instance of the server via running binaray is a child process
    pub fn spawn() -> Result<()> {
        std::process::Command::new(DAEMON_BINARY)
            .spawn()
            .context("Unable to start background instance using daemon binaray")
            .map(|_| ())
    }

    /// Pass args to running daemon
    pub fn execute(args: &[&str]) -> Result<()> {
        use std::io::Write;
        match std::os::unix::net::UnixStream::connect(DAEMON_SOCKET_PATH) {
            Ok(mut stream) => {
                stream.write_all(args.join(" ").as_str().as_ref())?;
                stream.flush().context("Fail to flush stream!")
            }
            Err(e) => bail!("Fail to execute {:#?}: {e}", args),
        }
    }
}

#[cfg(feature = "lua")]
use crate::util::mlua::LuaExtension;

#[cfg(feature = "lua")]
impl Daemon {
    /// Representation of daemon table in lua
    pub fn lua(lua: &mlua::Lua) -> mlua::Result<mlua::Table> {
        let table = lua.create_table()?;
        table.set("is_running", lua.create_function(Self::is_running)?)?;
        table.set("ensure", lua.create_function(Self::ensure)?)?;
        table.set("register", lua.create_function(Register::lua)?)?;
        table.set("drop", lua.create_function(Drop::lua)?)?;
        Ok(table)
    }

    /// Check if Daemon is running
    pub fn is_running(_: &mlua::Lua, _: ()) -> mlua::Result<bool> {
        match std::os::unix::net::UnixStream::connect(DAEMON_SOCKET_PATH) {
            Ok(stream) => Ok(stream.shutdown(std::net::Shutdown::Both).ok().is_some()),
            Err(_) => Ok(false),
        }
    }

    /// Ensure that daemon is currently running in background
    pub fn ensure(lua: &mlua::Lua, _: ()) -> mlua::Result<bool> {
        if Self::is_running(lua, ()).unwrap() {
            Ok(false)
        } else if Self::spawn().is_ok() {
            lua.info("Spawned Background Server")?;
            Ok(true)
        } else {
            panic!("Unable to spawn background server");
        }
    }
}
