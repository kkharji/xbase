use mlua::prelude::*;

pub(crate) trait NvimGlobal {
    fn vim(&self) -> LuaResult<LuaTable>;
}

impl NvimGlobal for Lua {
    fn vim(&self) -> LuaResult<LuaTable> {
        self.globals().get("vim")
    }
}

pub(crate) trait NvimNotify {
    fn notify<S: AsRef<str>>(&self, msg: S, level: usize) -> LuaResult<()>;
    fn trace<S: AsRef<str>>(&self, msg: S) -> LuaResult<()>;
    fn error<S: AsRef<str>>(&self, msg: S) -> LuaResult<()>;
    fn debug<S: AsRef<str>>(&self, msg: S) -> LuaResult<()>;
    fn warn<S: AsRef<str>>(&self, msg: S) -> LuaResult<()>;
    fn info<S: AsRef<str>>(&self, msg: S) -> LuaResult<()>;
}

impl NvimNotify for Lua {
    fn notify<S: AsRef<str>>(&self, msg: S, level: usize) -> LuaResult<()> {
        let msg = msg.as_ref();
        let notify: LuaFunction = self.vim()?.get("notify")?;
        notify.call::<_, ()>((format!("⋇ XBase: {msg}"), level))
    }

    fn trace<S: AsRef<str>>(&self, msg: S) -> Result<(), LuaError> {
        self.notify(msg, 0)
    }

    fn error<S: AsRef<str>>(&self, msg: S) -> Result<(), LuaError> {
        self.notify(msg, 4)
    }

    fn debug<S: AsRef<str>>(&self, msg: S) -> Result<(), LuaError> {
        self.notify(msg, 1)
    }

    fn warn<S: AsRef<str>>(&self, msg: S) -> Result<(), LuaError> {
        self.notify(msg, 3)
    }

    fn info<S: AsRef<str>>(&self, msg: S) -> LuaResult<()> {
        self.notify(msg, 2)
    }
}
