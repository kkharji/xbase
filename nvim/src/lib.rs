mod extensions;
mod tables;

use tables::Server;

use mlua::lua_module;
use mlua::prelude::*;

#[lua_module]
fn libxcodebase(lua: &Lua) -> LuaResult<LuaTable> {
    let server = lua.create_table()?;
    server.set("is_running", lua.create_function(Server::is_running)?)?;
    server.set("ensure", lua.create_function(Server::ensure)?)?;
    server.set("register", lua.create_function(Server::register_client)?)?;

    let commands = lua.create_table()?;

    let module = lua.create_table()?;
    module.set("commands", commands)?;
    module.set("server", server)?;
    Ok(module)
}
