//! Handle requests from neovim and manage dev workflow
mod message;
mod requests;
pub mod state;

mod client;
pub use client::Client;

#[cfg(feature = "daemon")]
mod nvim;

#[cfg(feature = "lua")]
use crate::util::mlua::LuaExtension;

pub use message::*;
pub use requests::*;

#[cfg(feature = "lua")]
use mlua::prelude::*;

#[cfg(feature = "daemon")]
pub use state::DaemonState;

#[cfg(feature = "lua")]
use std::{io::Write, net::Shutdown, os::unix::net::UnixStream, process::Command};

pub const DAEMON_SOCKET_PATH: &str = "/tmp/xcodebase-daemon.socket";
pub const DAEMON_BINARY: &str =
    "/Users/tami5/repos/neovim/xcodebase.nvim/target/debug/xcodebase-daemon";

/// Representation of daemon
pub struct Daemon;

#[cfg(feature = "lua")]
impl Daemon {
    /// Check if Daemon is running
    pub fn is_running(_: &Lua, _: ()) -> LuaResult<bool> {
        Ok(match UnixStream::connect(DAEMON_SOCKET_PATH) {
            Ok(stream) => stream.shutdown(Shutdown::Both).ok().is_some(),
            Err(_) => false,
        })
    }

    /// Ensure that daemon is currently running in background
    pub fn ensure(lua: &Lua, _: ()) -> LuaResult<bool> {
        if Self::is_running(lua, ()).unwrap() {
            Ok(false)
        } else if Command::new(DAEMON_BINARY).spawn().is_ok() {
            lua.info("Spawned Background Server")?;
            Ok(true)
        } else {
            panic!("Unable to spawn background server");
        }
    }

    /// Pass args to running daemon
    pub fn execute<I: Into<Request> + std::fmt::Debug>(_lua: &Lua, message: I) -> LuaResult<()> {
        let req: Request = message.into();
        let mut stream = UnixStream::connect(DAEMON_SOCKET_PATH)
            .map_err(|e| format!("Connect: {e} and execute: {:#?}", req))
            .to_lua_err()?;

        serde_json::to_vec(&req)
            .map(|value| stream.write_all(&value))
            .to_lua_err()??;

        stream.flush().to_lua_err()
    }
}
