use crate::types::*;
use crate::util::value_or_default;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, path::PathBuf};

#[cfg(feature = "neovim")]
use mlua::prelude::*;

/// Request to build a particular project
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuildRequest {
    pub root: PathBuf,
    pub settings: BuildSettings,
    #[serde(deserialize_with = "value_or_default")]
    pub direction: BufferDirection,
    pub ops: Operation,
}

/// Request to Run a particular project.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunRequest {
    pub root: PathBuf,
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
                root: table.get::<_, String>("root")?.into(),
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
        write!(f, "{}:Build:{}", self.root.display(), self.settings)
    }
}

#[cfg(feature = "neovim")]
impl<'a> FromLua<'a> for RunRequest {
    fn from_lua(value: LuaValue<'a>, _: &'a Lua) -> LuaResult<Self> {
        if let LuaValue::Table(table) = value {
            Ok(Self {
                root: table.get::<_, String>("root")?.into(),
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
            self.root.display(),
            self.device.name.as_ref().unwrap_or(&"Bin".to_string()),
            self.settings
        )
    }
}
