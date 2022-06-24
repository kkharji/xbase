//! mlua functions and extensions
use std::fmt::Display;

use mlua::prelude::*;
use tap::Pipe;

pub trait LuaExtension {
    fn print<S: Display>(&self, msg: &S);
    fn trace<S: Display>(&self, msg: &S);
    fn error<S: Display>(&self, msg: &S);
    fn debug<S: Display>(&self, msg: &S);
    fn warn<S: Display>(&self, msg: &S);
    fn info<S: Display>(&self, msg: &S);
    fn cwd(&self) -> LuaResult<String>;
    fn nvim_address(&self) -> LuaResult<String>;
    fn global_state(&self) -> LuaResult<LuaTable>;
}

fn log<S: Display>(lua: &Lua, level: &str, msg: &S) {
    // TODO: remove _XBASELOG hack to log from rust and lua
    lua.globals()
        .get::<_, LuaTable>("_XBASELOG")
        .and_then(|t| t.get::<_, LuaFunction>(level))
        .and_then(|f| f.call(msg.to_string().to_lua(lua)))
        .unwrap_or_else(|err| lua.print(&format!("UNABLE TO USE _XBASELOG!! {}", err.to_string())));
}

impl LuaExtension for Lua {
    fn print<S: Display>(&self, msg: &S) {
        self.globals()
            .get::<_, LuaFunction>("print")
            .unwrap()
            .call::<_, ()>(format!("{msg}").to_lua(self))
            .unwrap()
    }

    fn trace<S: Display>(&self, msg: &S) {
        log(self, "trace", msg)
    }

    fn error<S: Display>(&self, msg: &S) {
        log(self, "error", msg)
    }

    fn debug<S: Display>(&self, msg: &S) {
        log(self, "debug", msg)
    }

    fn warn<S: Display>(&self, msg: &S) {
        log(self, "warn", msg)
    }

    fn info<S: Display + Sized>(&self, msg: &S) {
        self.print(msg)
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

    fn nvim_address(&self) -> LuaResult<String> {
        let global = self.globals();
        match global.get::<_, LuaString>("__SERVERNAME") {
            Ok(v) => v,
            Err(_) => {
                let value = global
                    .get::<_, LuaTable>("vim")
                    .and_then(|v| v.get::<_, LuaTable>("v"))
                    .and_then(|v| v.get::<_, LuaString>("servername"))?;
                global.set("__SERVERNAME", value.clone())?;
                value
            }
        }
        .to_string_lossy()
        .to_string()
        .pipe(Ok)
    }
}
