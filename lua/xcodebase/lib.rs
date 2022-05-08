use mlua::prelude::*;
use xcodebase::daemon::*;

#[mlua::lua_module]
fn libxcodebase(lua: &Lua) -> LuaResult<LuaTable> {
    let is_running = lua.create_function(Daemon::is_running)?;
    let ensure = lua.create_function(Daemon::ensure)?;
    let register = lua.create_function(Register::request)?;
    let drop = lua.create_function(Drop::request)?;
    let build = lua.create_function(Build::request)?;
    let project_info = lua.create_function(ProjectInfo::request)?;

    lua.create_table_from([
        ("is_running", is_running),
        ("ensure", ensure),
        ("register", register),
        ("drop", drop),
        ("build", build),
        ("project_info", project_info),
    ])
}
