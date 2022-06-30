use mlua::{chunk, prelude::*};
use serde::{Deserialize, Serialize};
use std::cell::{Ref, RefMut};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct XBaseState {
    pub bufnr: usize,
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

            self.set_app_data(XBaseState { bufnr });
        }
        Ok(())
    }

    fn state(&self) -> LuaResult<std::cell::Ref<XBaseState>> {
        self.app_data_ref::<XBaseState>()
            .ok_or_else(|| LuaError::external("XBaseState is not set!"))
    }

    fn state_mut(&self) -> LuaResult<std::cell::RefMut<XBaseState>> {
        self.app_data_mut::<XBaseState>()
            .ok_or_else(|| LuaError::external("XBaseState is not set!"))
    }
    fn log_bufnr(&self) -> LuaResult<usize> {
        Ok(self.state()?.bufnr)
    }
}

impl<'lua> FromLua<'lua> for XBaseState {
    fn from_lua(value: LuaValue<'lua>, _lua: &'lua Lua) -> LuaResult<Self> {
        if let LuaValue::Table(table) = value {
            Ok(Self {
                bufnr: table.get("bufnr")?,
            })
        } else {
            Err(LuaError::external("Expected table, got something else"))
        }
    }
}

impl<'lua> ToLua<'lua> for XBaseState {
    fn to_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        let table = lua.create_table()?;
        table.set("bufnr", self.bufnr)?;
        Ok(LuaValue::Table(table))
    }
}
