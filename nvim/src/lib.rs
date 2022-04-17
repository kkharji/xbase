mod extensions;
mod tables;

use tables::Daemon;

use mlua::lua_module;
use mlua::prelude::*;

#[lua_module]
fn libxcodebase(lua: &Lua) -> LuaResult<LuaTable> {
    let server = lua.create_table()?;
    server.set("is_running", lua.create_function(Daemon::is_running)?)?;
    server.set("ensure", lua.create_function(Daemon::ensure)?)?;
    server.set("register", lua.create_function(Daemon::register_client)?)?;

    let commands = lua.create_table()?;

    let module = lua.create_table()?;
    module.set("commands", commands)?;
    module.set("server", server)?;
    Ok(module)
}
