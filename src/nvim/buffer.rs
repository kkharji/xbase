#[cfg(feature = "daemon")]
use super::NvimClient;
#[cfg(feature = "daemon")]
use crate::Result;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, strum::EnumString, Serialize, Deserialize)]
#[strum(ascii_case_insensitive)]
pub enum BufferDirection {
    Default,
    Vertical,
    Horizontal,
    TabEdit,
}

#[cfg(feature = "daemon")]
impl BufferDirection {
    pub fn to_nvim_command(&self, bufnr: i64) -> String {
        match self {
            // TOOD: support build log float as default
            BufferDirection::Default => format!("sbuffer {bufnr}"),
            BufferDirection::Vertical => format!("vert sbuffer {bufnr}"),
            BufferDirection::Horizontal => format!("sbuffer {bufnr}"),
            BufferDirection::TabEdit => format!("tabe {bufnr}"),
        }
    }

    pub async fn get_window_direction(
        nvim: &NvimClient,
        direction: Option<BufferDirection>,
        bufnr: i64,
    ) -> Result<String> {
        use std::str::FromStr;
        use tap::Pipe;

        if let Some(direction) = direction {
            return Ok(direction.to_nvim_command(bufnr));
        };

        match "return require'xbase.config'.values.default_log_buffer_direction"
            .pipe(|str| nvim.exec_lua(str, vec![]))
            .await?
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Unable to covnert value to string"))?
            .pipe(Self::from_str)
            .map(|d| d.to_nvim_command(bufnr))
        {
            Ok(open_command) => open_command,
            Err(e) => {
                tracing::error!("Unable to convert value to string {e}");
                Self::Horizontal.to_nvim_command(bufnr)
            }
        }
        .pipe(Ok)
    }
}

#[cfg(feature = "lua")]
use mlua::prelude::*;

#[cfg(feature = "lua")]
impl<'a> FromLua<'a> for BufferDirection {
    fn from_lua(lua_value: LuaValue<'a>, _lua: &'a Lua) -> LuaResult<Self> {
        use std::str::FromStr;
        use tap::Pipe;

        match lua_value {
            LuaValue::String(value) => value,
            _ => return Err(LuaError::external("Fail to deserialize Build")),
        }
        .to_string_lossy()
        .pipe(|s| Self::from_str(&s))
        .to_lua_err()
    }
}
