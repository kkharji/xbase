use crate::neovim::state::XBaseStateExt;
use mlua::{chunk, prelude::*};
use std::path::PathBuf;
use xbase_proto::MessageLevel;

pub trait NvimGlobal {
    fn vim(&self) -> LuaResult<LuaTable>;
    fn api(&self) -> LuaResult<LuaTable>;
    fn root(&self, root: Option<String>) -> LuaResult<PathBuf>;
    fn notify<S: AsRef<str>>(&self, msg: S, level: MessageLevel) -> LuaResult<()>;
    fn log<S: AsRef<str>>(&self, msg: S, level: MessageLevel) -> LuaResult<()>;
}

impl NvimGlobal for Lua {
    fn vim(&self) -> LuaResult<LuaTable> {
        self.globals().get("vim")
    }

    fn api(&self) -> LuaResult<LuaTable> {
        self.vim()?.get("api")
    }

    fn root(&self, root: Option<String>) -> LuaResult<PathBuf> {
        let root = match root {
            Some(root) => root,
            None => self
                .vim()?
                .get::<_, LuaTable>("loop")?
                .get::<_, LuaFunction>("cwd")?
                .call::<_, String>(())?,
        };
        Ok(PathBuf::from(root))
    }

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

    // TODO: Change line color based on level
    fn log<S: AsRef<str>>(&self, msg: S, level: MessageLevel) -> LuaResult<()> {
        let msg = msg.as_ref().trim();
        let user_level = self.state()?.log_level.clone() as u8;
        if msg.is_empty() || user_level > (level as u8) {
            return Ok(());
        }
        let bufnr = self.state()?.bufnr;
        let api = self.api()?;

        let set_lines: LuaFunction = api.get("nvim_buf_set_lines")?;
        let is_empty: bool = self
            .load(chunk! {
                return vim.api.nvim_buf_line_count($bufnr) == 1
                    and vim.api.nvim_buf_get_lines($bufnr, 0, 1, false)[1] == ""
            })
            .eval()?;

        set_lines.call((bufnr, if is_empty { 0 } else { -1 }, -1, false, vec![msg]))?;

        Ok(())
    }
}

pub trait NvimNotify: NvimGlobal {
    fn trace<S: AsRef<str>>(&self, msg: S) -> Result<(), LuaError> {
        self.notify(msg, MessageLevel::Trace)
    }

    fn debug<S: AsRef<str>>(&self, msg: S) -> Result<(), LuaError> {
        self.notify(msg, MessageLevel::Debug)
    }

    fn error<S: AsRef<str>>(&self, msg: S) -> Result<(), LuaError> {
        self.notify(msg, MessageLevel::Error)
    }

    fn warn<S: AsRef<str>>(&self, msg: S) -> Result<(), LuaError> {
        self.notify(msg, MessageLevel::Warn)
    }

    fn info<S: AsRef<str>>(&self, msg: S) -> LuaResult<()> {
        self.notify(msg, MessageLevel::Info)
    }
}

pub trait NvimLog: NvimGlobal {
    fn log_trace<S: AsRef<str>>(&self, msg: S) -> Result<(), LuaError> {
        self.log(msg, MessageLevel::Trace)
    }

    fn log_error<S: AsRef<str>>(&self, msg: S) -> Result<(), LuaError> {
        self.log(msg, MessageLevel::Error)
    }

    fn log_debug<S: AsRef<str>>(&self, msg: S) -> Result<(), LuaError> {
        self.log(msg, MessageLevel::Debug)
    }

    fn log_warn<S: AsRef<str>>(&self, msg: S) -> Result<(), LuaError> {
        self.log(msg, MessageLevel::Warn)
    }

    fn log_info<S: AsRef<str>>(&self, msg: S) -> LuaResult<()> {
        self.log(msg, MessageLevel::Info)
    }
}

impl NvimNotify for Lua {}
