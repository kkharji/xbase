use mlua::prelude::*;
use std::path::PathBuf;
use xbase_proto::MessageLevel;

pub trait NvimGlobal {
    fn vim(&self) -> LuaResult<LuaTable>;
    fn api(&self) -> LuaResult<LuaTable>;
    fn cwd(&self) -> LuaResult<PathBuf>;
    fn notify<S: AsRef<str>>(&self, msg: S, level: MessageLevel) -> LuaResult<()>;
    fn log<S: AsRef<str>>(&self, msg: S, level: MessageLevel) -> LuaResult<()>;
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
