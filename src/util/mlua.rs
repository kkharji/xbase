use mlua::prelude::*;

pub trait LuaExtension {
    fn print(&self, msg: &str);
    fn trace(&self, msg: &str) -> LuaResult<()>;
    fn error(&self, msg: &str) -> LuaResult<()>;
    fn debug(&self, msg: &str) -> LuaResult<()>;
    fn warn(&self, msg: &str) -> LuaResult<()>;
    fn info(&self, msg: &str) -> LuaResult<()>;
}

fn log(lua: &Lua, level: &str, msg: &str) -> LuaResult<()> {
    // lua.load(mlua::chunk::chunk!(require("xcodebase.log").$level($msg))).exec()
    // TODO: remove _XCODEBASELOG hack to log from rust and lua
    lua.globals()
        .get::<_, LuaTable>("_XCODEBASELOG")?
        .get::<_, LuaFunction>(level)?
        .call::<_, ()>(msg.to_lua(lua))
}

impl LuaExtension for Lua {
    fn print(&self, msg: &str) {
        self.globals()
            .get::<_, LuaFunction>("print")
            .unwrap()
            .call::<_, ()>(format!("xcodebase: {}", msg).to_lua(self))
            .unwrap()
    }

    fn trace(&self, msg: &str) -> LuaResult<()> {
        log(self, "trace", msg)
    }

    fn error(&self, msg: &str) -> LuaResult<()> {
        log(self, "error", msg)
    }

    fn debug(&self, msg: &str) -> LuaResult<()> {
        log(self, "debug", msg)
    }

    fn warn(&self, msg: &str) -> LuaResult<()> {
        log(self, "warn", msg)
    }

    fn info(&self, msg: &str) -> LuaResult<()> {
        log(self, "info", msg)
    }
}
