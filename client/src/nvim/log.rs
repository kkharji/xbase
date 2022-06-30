use super::NvimGlobal;
use crate::nvim::state::XBaseStateExt;
use mlua::prelude::*;
use xbase_proto::MessageLevel;

pub trait NvimLog {
    fn log<S: AsRef<str>>(&self, msg: S, level: MessageLevel) -> LuaResult<()>;
}

impl NvimLog for Lua {
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
            .load(mlua::chunk! {
                return vim.api.nvim_buf_line_count($bufnr) == 1
                    and vim.api.nvim_buf_get_lines($bufnr, 0, 1, false)[1] == ""
            })
            .eval()?;

        set_lines.call((bufnr, if is_empty { 0 } else { -1 }, -1, false, vec![msg]))?;

        Ok(())
    }
}
