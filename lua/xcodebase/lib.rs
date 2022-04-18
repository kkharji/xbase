use mlua::lua_module;
use mlua::prelude::*;
use xcodebase::Daemon;

#[lua_module]
fn libxcodebase(lua: &Lua) -> LuaResult<LuaTable> {
    let commands = lua.create_table()?;
    let module = lua.create_table()?;
    module.set("commands", commands)?;
    module.set("daemon", Daemon::lua(lua)?)?;
    Ok(module)
}
