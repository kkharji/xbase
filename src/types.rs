use std::path::PathBuf;

#[cfg(feature = "lua")]
use mlua::prelude::*;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

mod project;
pub use project::*;

pub type Root = PathBuf;

#[derive(Hash, Clone, Debug, Default, Serialize, Deserialize, strum::Display, PartialEq, Eq)]
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

#[derive(
    Hash, Default, Clone, Debug, Serialize, Deserialize, Display, EnumString, PartialEq, Eq,
)]
pub enum WatchType {
    #[default]
    Build,
    Run,
}

/// Fields required to build a project
#[derive(Clone, Default, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
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
    /// Watch type
    pub watch_type: WatchType,
}

impl std::fmt::Display for BuildConfiguration {
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

#[cfg(feature = "lua")]
impl<'a> FromLua<'a> for BuildConfiguration {
    fn from_lua(lua_value: LuaValue<'a>, _lua: &'a Lua) -> LuaResult<Self> {
        use std::str::FromStr;
        if let LuaValue::Table(table) = lua_value {
            let watch_type = match table.get::<_, Option<String>>("watch_type")? {
                Some(w) => WatchType::from_str(&w).to_lua_err()?,
                None => WatchType::Build,
            };

            Ok(Self {
                sysroot: table.get("sysroot")?,
                target: table.get("target")?,
                configuration: table.get("configuration")?,
                scheme: table.get("scheme")?,
                watch_type,
            })
        } else {
            Ok(BuildConfiguration::default())
        }
    }
}

#[derive(Clone, Default, Debug, Deserialize, Serialize, Hash, PartialEq, Eq)]
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
            root: LuaExtension::cwd(lua).map(PathBuf::from)?,
        })
    }
}
