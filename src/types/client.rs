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
impl<'a> mlua::FromLua<'a> for Client {
    fn from_lua(_lua_value: mlua::Value<'a>, lua: &'a mlua::Lua) -> mlua::Result<Self> {
        use crate::util::mlua::LuaExtension;
        Ok(Self {
            pid: std::process::id() as i32,
            root: LuaExtension::cwd(lua).map(std::path::PathBuf::from)?,
        })
    }
}
