use super::{NvimLog, NvimNotify};
use mlua::prelude::*;
use std::path::PathBuf;
use tap::Pipe;
use xbase_proto::MessageLevel;

pub trait NvimGlobal: NvimLog + NvimNotify {
    fn vim(&self) -> LuaResult<LuaTable>;
    fn api(&self) -> LuaResult<LuaTable>;
    fn root(&self, root: Option<String>) -> LuaResult<PathBuf>;
    fn trace<S: AsRef<str>>(&self, msg: S) -> Result<(), LuaError>;
    fn debug<S: AsRef<str>>(&self, msg: S) -> Result<(), LuaError>;
    fn error<S: AsRef<str>>(&self, msg: S) -> Result<(), LuaError>;
    fn warn<S: AsRef<str>>(&self, msg: S) -> Result<(), LuaError>;
    fn info<S: AsRef<str>>(&self, msg: S) -> LuaResult<()>;
    fn log_trace<S: AsRef<str>>(&self, msg: S) -> Result<(), LuaError>;
    fn log_error<S: AsRef<str>>(&self, msg: S) -> Result<(), LuaError>;
    fn log_debug<S: AsRef<str>>(&self, msg: S) -> Result<(), LuaError>;
    fn log_warn<S: AsRef<str>>(&self, msg: S) -> Result<(), LuaError>;
    fn log_info<S: AsRef<str>>(&self, msg: S) -> LuaResult<()>;
}

impl NvimGlobal for Lua {
    fn vim(&self) -> LuaResult<LuaTable> {
        self.globals().get("vim")
    }

    fn api(&self) -> LuaResult<LuaTable> {
        self.vim()?.get("api")
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
