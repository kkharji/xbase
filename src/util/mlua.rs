//! mlua functions and extensions
use mlua::prelude::*;

pub trait LuaExtension {
    fn print(&self, msg: &str);
    fn trace(&self, msg: &str) -> LuaResult<()>;
    fn error(&self, msg: &str) -> LuaResult<()>;
    fn debug(&self, msg: &str) -> LuaResult<()>;
    fn warn(&self, msg: &str) -> LuaResult<()>;
    fn info(&self, msg: &str) -> LuaResult<()>;
    fn cwd(&self) -> LuaResult<String>;
    fn global_state(&self) -> LuaResult<LuaTable>;
}

fn log(lua: &Lua, level: &str, msg: &str) -> LuaResult<()> {
    // lua.load(mlua::chunk::chunk!(require("xbase.log").$level($msg))).exec()
    // TODO: remove _XBASELOG hack to log from rust and lua
    lua.globals()
        .get::<_, LuaTable>("_XBASELOG")?
        .get::<_, LuaFunction>(level)?
        .call::<_, ()>(msg.to_lua(lua))
}

impl LuaExtension for Lua {
    fn print(&self, msg: &str) {
        self.globals()
            .get::<_, LuaFunction>("print")
            .unwrap()
            .call::<_, ()>(format!("xbase: {}", msg).to_lua(self))
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

    fn global_state(&self) -> LuaResult<LuaTable> {
        self.globals()
            .get::<_, LuaTable>("vim")?
            .get::<_, LuaTable>("g")?
            .get::<_, LuaTable>("xbase")
    }

    fn cwd(&self) -> LuaResult<String> {
        self.globals()
            .get::<_, LuaTable>("vim")?
            .get::<_, LuaTable>("loop")?
            .get::<_, LuaFunction>("cwd")?
            .call::<_, String>(())
    }
}
