use super::NvimGlobal;
use super::NvimLogBuffer;
use mlua::prelude::*;
use serde::{Deserialize, Serialize};
use xbase_proto::Result;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NvimState {
    buffer: NvimLogBuffer,
}

impl NvimState {
    pub fn new(lua: &Lua) -> Result<Self> {
        let lua: &'static Lua = unsafe { std::mem::transmute(lua) };
        Ok(Self {
            buffer: NvimLogBuffer::new(lua)?,
        })
    }
}

impl<'lua> FromLua<'lua> for NvimState {
    fn from_lua(value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        if let LuaValue::Table(table) = value {
            Ok(Self {
                buffer: table.get("buffer")?,
            })
        } else {
            let state = Self::new(lua)?;
            lua.set_state(state.clone())?;
            Ok(state)
        }
    }
}

impl<'lua> ToLua<'lua> for NvimState {
    fn to_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        let table = lua.create_table()?;
        table.set("buffer", self.buffer)?;
        Ok(LuaValue::Table(table))
    }
}