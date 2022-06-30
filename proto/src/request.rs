use crate::types::*;
use crate::util::value_or_default;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[cfg(feature = "neovim")]
use mlua::prelude::*;

/// Request to build a particular project
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuildRequest {
    pub client: Client,
    pub settings: BuildSettings,
    #[serde(deserialize_with = "value_or_default")]
    pub direction: BufferDirection,
    pub ops: Operation,
}

/// Request to Run a particular project.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunRequest {
    pub client: Client,
    pub settings: BuildSettings,
    #[serde(deserialize_with = "value_or_default")]
    pub device: DeviceLookup,
    #[serde(deserialize_with = "value_or_default")]
    pub direction: BufferDirection,
    #[serde(deserialize_with = "value_or_default")]
    pub ops: Operation,
}

#[cfg(feature = "neovim")]
impl<'a> FromLua<'a> for BuildRequest {
    fn from_lua(value: LuaValue<'a>, _: &'a Lua) -> LuaResult<Self> {
        if let LuaValue::Table(table) = value {
            Ok(Self {
                client: table.get("client")?,
                settings: table.get("settings")?,
                direction: table.get("direction")?,
                ops: table.get("ops")?,
            })
        } else {
            Err(LuaError::external("Expected a table for BuildRequest"))
        }
    }
}

impl Display for BuildRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:Build:{}", self.client.root.display(), self.settings)
    }
}

#[cfg(feature = "neovim")]
impl<'a> FromLua<'a> for RunRequest {
    fn from_lua(value: LuaValue<'a>, _: &'a Lua) -> LuaResult<Self> {
        if let LuaValue::Table(table) = value {
            Ok(Self {
                client: table.get("client")?,
                settings: table.get("settings")?,
                direction: table.get("direction")?,
                device: table.get("device")?,
                ops: table.get("ops")?,
            })
        } else {
            Err(LuaError::external("Expected a table for BuildRequest"))
        }
    }
}

impl Display for RunRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:Run:{}:{}",
            self.client.root.display(),
            self.device.name.as_ref().unwrap_or(&"Bin".to_string()),
            self.settings
        )
    }
}
