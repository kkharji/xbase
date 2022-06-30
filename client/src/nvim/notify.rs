use super::NvimGlobal;
use mlua::prelude::*;
use xbase_proto::MessageLevel;

pub trait NvimNotify {
    fn notify<S: AsRef<str>>(&self, msg: S, level: MessageLevel) -> LuaResult<()>;
}

impl NvimNotify for Lua {
    fn notify<S: AsRef<str>>(&self, msg: S, level: MessageLevel) -> LuaResult<()> {
        if msg.as_ref().trim().is_empty() {
            return Ok(());
        }

        let msg = msg.as_ref();
        let vim = self.vim()?;
        let level = level as u8;
        let opts = self.create_table_from([("title", "XBase")])?;
        let args = (msg, level, opts);

        // NOTE: Plugins like nvim-notify sets notify to metatable
        match vim.get::<_, LuaFunction>("notify") {
            Ok(f) => f.call(args),
            Err(_) => vim.get::<_, LuaTable>("notify")?.call(args),
        }
    }
}
