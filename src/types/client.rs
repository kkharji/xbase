use super::Root;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct Client {
    pub pid: i32,
    pub root: Root,
}

impl Client {
    pub fn abbrev_root(&self) -> String {
        self.root
            .strip_prefix(self.root.ancestors().nth(2).unwrap())
            .unwrap()
            .display()
            .to_string()
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
