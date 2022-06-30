use mlua::{chunk, prelude::*};
use std::path::PathBuf;
use tap::Pipe;
use xbase_proto::MessageLevel;

pub trait NvimGlobal {
    fn vim(&self) -> LuaResult<LuaTable>;
    fn api(&self, fn_name: &'static str) -> LuaResult<LuaFunction>;
    fn root(&self, root: Option<String>) -> LuaResult<PathBuf>;
    fn info<S: AsRef<str>>(&self, msg: S) -> LuaResult<()>;
    fn log<S: AsRef<str>>(&self, msg: S, level: MessageLevel) -> Result<(), LuaError>;
    fn notify<S: AsRef<str>>(&self, msg: S, level: MessageLevel) -> LuaResult<()>;
}

impl NvimGlobal for Lua {
    fn vim(&self) -> LuaResult<LuaTable> {
        self.globals().get("vim")
    }

    fn api(&self, fn_name: &'static str) -> LuaResult<LuaFunction> {
        self.vim()?.get::<_, LuaTable>("api")?.get(fn_name)
    }

    fn root(&self, root: Option<String>) -> LuaResult<PathBuf> {
        match root {
            Some(root) => root,
            None => {
                let vim = self.vim()?.get::<_, LuaTable>("loop")?;
                vim.get::<_, LuaFunction>("cwd")?.call(())?
            }
        }
        .pipe(PathBuf::from)
        .pipe(Ok)
    }

    fn info<S: AsRef<str>>(&self, msg: S) -> LuaResult<()> {
        self.notify(msg, MessageLevel::Info)
    }

    fn log<S: AsRef<str>>(&self, msg: S, level: MessageLevel) -> Result<(), LuaError> {
        let log: LuaFunction = self.load(chunk!(return require("xbase.log").log)).eval()?;
        log.call::<_, ()>((msg.as_ref(), level as u8))
    }
    fn notify<S: AsRef<str>>(&self, msg: S, level: MessageLevel) -> LuaResult<()> {
        let msg = msg.as_ref();
        if msg.trim().is_empty() {
            return Ok(());
        }

        if matches!(level, MessageLevel::Success) {
            return self
                .api("nvim_echo")?
                .call(([[msg, "healthSuccess"]], true, Vec::<u8>::new()));
        }

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
