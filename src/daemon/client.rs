use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Client {
    pub pid: i32,
    pub root: String,
}

#[cfg(feature = "lua")]
impl<'a> mlua::FromLua<'a> for Client {
    fn from_lua(_lua_value: mlua::Value<'a>, lua: &'a mlua::Lua) -> mlua::Result<Self> {
        Ok(Self {
            pid: std::process::id() as i32,
            root: crate::util::mlua::LuaExtension::cwd(lua)?,
        })
    }
}
