#[cfg(feature = "daemon")]
use crate::util::fs::get_dirname_dir_root;

use super::Root;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct Client {
    pub pid: i32,
    pub root: Root,
    pub address: String,
}

#[cfg(feature = "daemon")]
impl Client {
    pub fn abbrev_root(&self) -> String {
        get_dirname_dir_root(&self.root).unwrap_or_default()
    }
}

#[cfg(feature = "lua")]
use {crate::util::mlua::LuaExtension, mlua::prelude::*, tap::Pipe};

#[cfg(feature = "lua")]
impl Client {
    /// Derive client from lua value
    /// lua value can:
    /// - Client key with table value within it a key with "root"
    /// - Client key with string value representing "root"
    /// If value is none, then current working directory will be used
    /// lua value can either be a table with client key being a string
    pub fn derive(lua: &Lua, value: Option<LuaValue>) -> LuaResult<Self> {
        let root = match value {
            Some(LuaValue::Table(ref table)) => table.get("root")?,
            Some(LuaValue::String(ref root)) => root.to_string_lossy().to_string(),
            _ => lua.cwd()?,
        };
        Self {
            pid: std::process::id() as i32,
            address: lua.nvim_address()?,
            root: root.into(),
        }
        .pipe(Ok)
    }
}

#[cfg(feature = "lua")]
impl<'a> mlua::FromLua<'a> for Client {
    fn from_lua(value: mlua::Value<'a>, lua: &'a mlua::Lua) -> mlua::Result<Self> {
        Self::derive(
            lua,
            match value {
                LuaValue::Nil => None,
                _ => Some(value),
            },
        )
    }
}
