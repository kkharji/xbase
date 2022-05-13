use mlua::{lua_module, prelude::*};
use std::{net::Shutdown, os::unix::net::UnixStream, process::Command};
use xcodebase::{constants::*, daemon::*, util::mlua::LuaExtension};

macro_rules! fun {
    ($t:ident, $lua:ident) => {
        $lua.create_function($t::request)?
    };
    ($t:ident, $fun:ident, $lua:ident) => {
        $lua.create_function($t::$fun)?
    };
}

#[lua_module]
fn libxcodebase(l: &Lua) -> LuaResult<LuaTable> {
    l.create_table_from([
        ("is_running", l.create_function(is_running)?),
        ("ensure", l.create_function(ensure)?),
        ("register", fun!(Register, l)),
        ("drop", fun!(Drop, l)),
        ("build", fun!(Build, l)),
        ("watch_target", fun!(WatchTarget, l)),
        ("project_info", fun!(ProjectInfo, l)),
    ])
}

/// Check if Daemon is running
pub fn is_running(_: &Lua, _: ()) -> LuaResult<bool> {
    Ok(match UnixStream::connect(DAEMON_SOCKET_PATH) {
        Ok(stream) => stream.shutdown(Shutdown::Both).ok().is_some(),
        Err(_) => false,
    })
}

/// Ensure that daemon is currently running in background
pub fn ensure(lua: &Lua, _: ()) -> LuaResult<bool> {
    if is_running(lua, ()).unwrap() {
        Ok(false)
    } else if Command::new(DAEMON_BINARY).spawn().is_ok() {
        lua.info("Spawned Background Server")?;
        Ok(true)
    } else {
        panic!("Unable to spawn background server");
    }
}
