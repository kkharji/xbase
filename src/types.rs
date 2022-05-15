use std::{fmt::Display, path::PathBuf};

#[cfg(feature = "lua")]
use mlua::prelude::*;
use serde::{Deserialize, Serialize};

mod client;
mod project;

pub use client::*;
pub use project::*;

pub type Root = PathBuf;

/// Fields required to build a project
#[derive(Clone, Default, Debug, Serialize, Deserialize, Hash)]
pub struct BuildConfiguration {
    /// TODO(nvim): make build config sysroot default to tmp in auto-build
    pub sysroot: Option<String>,
    /// Target to build
    pub target: String,
    /// Configuration to build with, default Debug
    #[serde(default)]
    pub configuration: XConfiguration,
    /// Scheme to build with
    pub scheme: Option<String>,
}

#[cfg(feature = "lua")]
impl<'a> FromLua<'a> for BuildConfiguration {
    fn from_lua(lua_value: LuaValue<'a>, _lua: &'a Lua) -> LuaResult<Self> {
        if let LuaValue::Table(table) = lua_value {
            Ok(Self {
                sysroot: table.get("sysroot")?,
                target: table.get("target")?,
                configuration: table.get("configuration")?,
                scheme: table.get("scheme")?,
            })
        } else {
            Ok(BuildConfiguration::default())
        }
    }
}

impl Display for BuildConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "xcodebuild")?;
        write!(f, " -configuration {}", self.configuration)?;

        if let Some(ref sysroot) = self.sysroot {
            write!(f, " -sysroot {sysroot}")?;
        }
        if let Some(ref scheme) = self.scheme {
            write!(f, " -scheme {scheme}")?;
        }
        write!(f, " -target {}", self.target)?;
        Ok(())
    }
}

#[derive(Hash, Clone, Debug, Default, Serialize, Deserialize, strum::Display)]
#[serde(untagged)]
pub enum XConfiguration {
    #[default]
    Debug,
    Release,
    Custom(String),
}

#[cfg(feature = "lua")]
impl<'a> FromLua<'a> for XConfiguration {
    fn from_lua(lua_value: LuaValue<'a>, _lua: &'a Lua) -> LuaResult<Self> {
        if let LuaValue::String(config) = lua_value {
            let value = config.to_str()?;
            Ok(match value {
                "debug" | "Debug" => Self::Debug,
                "release" | "Release" => Self::Release,
                _ => Self::Custom(value.to_string()),
            })
        } else if matches!(lua_value, LuaValue::Nil) {
            Ok(Self::Debug)
        } else {
            Err(LuaError::external("Expected a table got XConfiguration"))
        }
    }
}
