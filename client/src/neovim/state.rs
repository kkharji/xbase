use mlua::{chunk, prelude::*};
use serde::{Deserialize, Serialize};
use std::cell::{Ref, RefMut};
use xbase_proto::MessageLevel;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct XBaseState {
    pub bufnr: usize,
    pub log_level: MessageLevel,
}

pub trait XBaseStateExt {
    fn setup_state(&self) -> LuaResult<()>;
    fn state(&self) -> LuaResult<Ref<XBaseState>>;
    fn log_bufnr(&self) -> LuaResult<usize>;
    fn state_mut(&self) -> LuaResult<RefMut<XBaseState>>;
}

impl XBaseStateExt for Lua {
    fn setup_state(&self) -> LuaResult<()> {
        // NOTE: Is it better to have a log buffer to every project root?
        if self.state().is_err() {
            let bufnr = self
                .load(chunk! {
                    local bufnr = vim.api.nvim_create_buf(false, true)
                    vim.api.nvim_buf_set_name(bufnr, "[XBase Logs]")
                    vim.api.nvim_buf_set_option(bufnr, "filetype", "xcodebuildlog");
                    vim.api.nvim_create_autocmd({"BufEnter"}, {
                        buffer = bufnr,
                        command = "setlocal nonumber norelativenumber scrolloff=3"
                    });
                    return bufnr
                })
                .eval::<usize>()?;

            let log_level: String = self
                .globals()
                .get::<_, LuaTable>("_XBASECONFIG")?
                .get("log_level")?;

            self.set_app_data(XBaseState {
                bufnr,
                log_level: match log_level.as_str() {
                    "info" => MessageLevel::Info,
                    "error" => MessageLevel::Error,
                    "warn" => MessageLevel::Warn,
                    "trace" => MessageLevel::Trace,
                    "debug" => MessageLevel::Debug,
                    _ => MessageLevel::Info,
                },
            });
        }
        Ok(())
    }

    fn state(&self) -> LuaResult<std::cell::Ref<XBaseState>> {
        self.app_data_ref::<XBaseState>()
            .ok_or_else(|| LuaError::external("XBaseState is not set!"))
    }

    fn log_bufnr(&self) -> LuaResult<usize> {
        Ok(self.state()?.bufnr)
    }
    fn state_mut(&self) -> LuaResult<std::cell::RefMut<XBaseState>> {
        self.app_data_mut::<XBaseState>()
            .ok_or_else(|| LuaError::external("XBaseState is not set!"))
    }
}
