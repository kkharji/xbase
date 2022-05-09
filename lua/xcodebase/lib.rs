use mlua::prelude::*;
use xcodebase::daemon::*;

macro_rules! fun {
    ($t:ident, $lua:ident) => {
        $lua.create_function($t::request)?
    };
    ($t:ident, $fun:ident, $lua:ident) => {
        $lua.create_function($t::$fun)?
    };
}

#[mlua::lua_module]
fn libxcodebase(l: &Lua) -> LuaResult<LuaTable> {
    l.create_table_from([
        ("is_running", fun!(Daemon, is_running, l)),
        ("ensure", fun!(Daemon, ensure, l)),
        ("register", fun!(Register, l)),
        ("drop", fun!(Drop, l)),
        ("build", fun!(Build, l)),
        ("watch_start", fun!(WatchStart, l)),
        ("watch_stop", fun!(WatchStop, l)),
        ("project_info", fun!(ProjectInfo, l)),
    ])
}
