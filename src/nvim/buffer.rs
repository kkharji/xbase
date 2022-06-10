use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, strum::EnumString, Serialize, Deserialize)]
#[strum(ascii_case_insensitive)]
pub enum BufferDirection {
    Default,
    Vertical,
    Horizontal,
    TabEdit,
}

impl Default for BufferDirection {
    fn default() -> Self {
        Self::Default
    }
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
