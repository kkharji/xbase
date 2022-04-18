use crate::extensions::LuaExt;

use mlua::prelude::*;
use xcodebase::{daemon, Register, UnRegister};

pub struct Daemon;

impl Daemon {
    pub fn is_running(_: &Lua, _: ()) -> LuaResult<bool> {
        Ok(daemon::is_running())
    }

    pub fn ensure(lua: &Lua, _: ()) -> LuaResult<bool> {
        if daemon::is_running() {
            return Ok(false);
        }

        if daemon::spawn().is_ok() {
            lua.info("Spawned Background Server")?;
            return Ok(true);
        }

        panic!("Unable to spawn background server");
    }
}

pub struct DaemonClient;
impl DaemonClient {
    pub fn register(lua: &Lua, (pid, root): (i32, String)) -> LuaResult<()> {
        lua.trace(&format!("Added (pid: {pid} cwd: {root})"))?;
        Register::request(pid, root).map_err(LuaError::external)
    }

    pub fn unregister(lua: &Lua, (pid, root): (i32, String)) -> LuaResult<()> {
        lua.trace(&format!("Removed (pid: {pid} cwd: {root})"))?;
        UnRegister::request(pid, root).map_err(LuaError::external)
    }
}
