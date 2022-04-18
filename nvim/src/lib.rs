mod extensions;
mod tables;

use tables::{Daemon, DaemonClient};

use mlua::lua_module;
use mlua::prelude::*;

#[lua_module]
fn libxcodebase(lua: &Lua) -> LuaResult<LuaTable> {
    let daemon = lua.create_table()?;
    daemon.set("is_running", lua.create_function(Daemon::is_running)?)?;
    daemon.set("ensure", lua.create_function(Daemon::ensure)?)?;
    daemon.set("register", lua.create_function(DaemonClient::register)?)?;
    daemon.set("unregister", lua.create_function(DaemonClient::unregister)?)?;

    let commands = lua.create_table()?;

    let module = lua.create_table()?;
    module.set("commands", commands)?;
    module.set("daemon", daemon)?;
    Ok(module)
}
