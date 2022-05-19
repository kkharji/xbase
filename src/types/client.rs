#[cfg(feature = "daemon")]
use crate::util::fs::get_dirname_dir_root;

use super::Root;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct Client {
    pub pid: i32,
    pub root: Root,
}

#[cfg(feature = "daemon")]
impl Client {
    pub fn abbrev_root(&self) -> String {
        get_dirname_dir_root(&self.root).unwrap_or_default()
    }
}

#[cfg(feature = "lua")]
use {crate::util::mlua::LuaExtension, mlua::prelude::*};
#[cfg(feature = "lua")]
impl Client {
    pub fn derive(lua: &Lua, value: Option<LuaValue>) -> LuaResult<Self> {
        let is_provided = value.is_some();
        let root = match value {
            Some(LuaValue::Table(ref table)) => table
                .get::<_, LuaTable>("client")
                .map(|t| t.get("root"))
                .flatten(),
            Some(LuaValue::String(ref root)) => Ok(root.to_string_lossy().to_string()),
            _ => lua.cwd(),
        }
        .map(std::path::PathBuf::from);

        Ok(Self {
            pid: std::process::id() as i32,
            root: match root {
                Ok(v) => v,
                Err(_) => {
                    if is_provided {
                        let msg = format!(
                            "Unable to get current working directory from value! {value:?}"
                        );

                        return Err(LuaError::external(msg));
                    } else {
                        return Err(LuaError::external(
                            "Unable to get current working directory!",
                        ));
                    }
                }
            },
        })
    }
}

#[cfg(feature = "lua")]
impl<'a> mlua::FromLua<'a> for Client {
    fn from_lua(value: mlua::Value<'a>, lua: &'a mlua::Lua) -> mlua::Result<Self> {
        Self::derive(lua, Some(value))
    }
}
