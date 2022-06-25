use serde::{Deserialize, Serialize};
use std::{fmt::Display, path::PathBuf};
use strum::{Display as EnumDisplay, EnumString};

#[cfg(feature = "neovim")]
use mlua::prelude::*;

/// Client data
#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct Client {
    pub pid: i32,
    pub root: PathBuf,
    pub address: String,
}

#[cfg(feature = "neovim")]
impl<'a> FromLua<'a> for Client {
    fn from_lua(value: LuaValue<'a>, lua: &'a Lua) -> LuaResult<Self> {
        Self::new(lua, {
            if let LuaValue::String(ref root) = value {
                Some(root.to_string_lossy().to_string())
            } else {
                None
            }
        })
    }
}

/// Build Configuration to run
#[derive(Clone, Debug, Serialize, Deserialize, EnumDisplay, EnumString)]
#[serde(untagged)]
pub enum BuildConfiguration {
    Debug,
    Release,
    Custom(String),
}

#[cfg(feature = "neovim")]
impl<'a> FromLua<'a> for BuildConfiguration {
    fn from_lua(value: LuaValue<'a>, _: &'a Lua) -> LuaResult<Self> {
        use std::str::FromStr;
        if let LuaValue::String(ref value) = value {
            Self::from_str(value.to_str()?).to_lua_err()
        } else {
            Err(LuaError::external(
                "Expected a string value for BuildConfiguration",
            ))
        }
    }
}

/// Operation
///
/// Should request be executed once, stoped (if watched) or start new watch service?
#[derive(Clone, Debug, Serialize, Deserialize, EnumDisplay, EnumString)]
#[serde(untagged)]
pub enum Operation {
    Watch,
    Stop,
    Once,
}

#[cfg(feature = "neovim")]
impl<'a> FromLua<'a> for Operation {
    fn from_lua(value: LuaValue<'a>, _: &'a Lua) -> LuaResult<Self> {
        use std::str::FromStr;
        if let LuaValue::String(value) = value {
            let value = value.to_string_lossy();
            Self::from_str(&*value).to_lua_err()
        } else {
            Ok(Operation::default())
        }
    }
}

/// Fields required to build a project
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuildSettings {
    /// Target to build
    pub target: String,
    /// Configuration to build with, default Debug
    pub configuration: BuildConfiguration,
    /// Scheme to build with
    pub scheme: Option<String>,
}

#[cfg(feature = "neovim")]
impl<'a> FromLua<'a> for BuildSettings {
    fn from_lua(value: LuaValue<'a>, _: &'a Lua) -> LuaResult<Self> {
        if let LuaValue::Table(table) = value {
            Ok(Self {
                target: table.get("target")?,
                configuration: table.get("configuration")?,
                scheme: table.get("scheme")?,
            })
        } else {
            Err(LuaError::external(
                "Expected a table value for BuildSettings",
            ))
        }
    }
}

/// Log Buffer open direction
#[derive(Clone, Debug, strum::EnumString, Serialize, Deserialize)]
#[strum(ascii_case_insensitive)]
#[serde(untagged)]
pub enum BufferDirection {
    Default,
    Vertical,
    Horizontal,
    TabEdit,
}

#[cfg(feature = "neovim")]
impl<'a> FromLua<'a> for BufferDirection {
    fn from_lua(value: LuaValue<'a>, _: &'a Lua) -> LuaResult<Self> {
        use std::str::FromStr;
        if let LuaValue::String(value) = value {
            let value = value.to_string_lossy();
            Self::from_str(&*value).to_lua_err()
        } else {
            Ok(Self::Default)
        }
    }
}

/// Device Lookup information to run built project with
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct DeviceLookup {
    pub name: Option<String>,
    pub udid: Option<String>,
}

#[cfg(feature = "neovim")]
impl<'a> FromLua<'a> for DeviceLookup {
    fn from_lua(value: LuaValue<'a>, _: &'a Lua) -> LuaResult<Self> {
        if let LuaValue::Table(table) = value {
            Ok(Self {
                name: table.get("name").ok(),
                udid: table.get("udid").ok(),
            })
        } else {
            Ok(Self::default())
        }
    }
}

impl Default for BufferDirection {
    fn default() -> Self {
        Self::Default
    }
}

impl Default for Operation {
    fn default() -> Self {
        Self::Once
    }
}

impl Display for BuildSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "-configuration {}", self.configuration)?;

        if let Some(ref scheme) = self.scheme {
            write!(f, " -scheme {scheme}")?;
        }
        write!(f, " -target {}", self.target)?;
        Ok(())
    }
}
impl BuildSettings {
    pub fn to_args(&self) -> Vec<String> {
        self.to_string()
            .split_whitespace()
            .map(ToString::to_string)
            .collect::<Vec<String>>()
    }
}

impl Operation {
    /// Returns `true` if the request kind is [`Watch`].
    ///
    /// [`Watch`]: RequestKind::Watch
    #[must_use]
    pub fn is_watch(&self) -> bool {
        matches!(self, Self::Watch)
    }

    /// Returns `true` if the request kind is [`WatchStop`].
    ///
    /// [`WatchStop`]: RequestKind::WatchStop
    #[must_use]
    pub fn is_stop(&self) -> bool {
        matches!(self, Self::Stop)
    }

    /// Returns `true` if the request kind is [`Once`].
    ///
    /// [`Once`]: RequestKind::Once
    #[must_use]
    pub fn is_once(&self) -> bool {
        matches!(self, Self::Once)
    }
}

impl Client {
    #[cfg(feature = "neovim")]
    pub fn new(lua: &Lua, root: Option<String>) -> LuaResult<Self> {
        use crate::util::address;
        use crate::util::cwd;
        Ok(Self {
            pid: std::process::id() as i32,
            address: address(lua)?,
            root: if let Some(v) = root { v } else { cwd(lua)? }.into(),
        })
    }

    pub fn abbrev_root(&self) -> String {
        let abbr = || {
            let path = &self.root;
            Some(
                path.strip_prefix(path.ancestors().nth(3)?)
                    .ok()?
                    .display()
                    .to_string(),
            )
        };

        abbr().unwrap_or_default()
    }
}

impl BufferDirection {
    pub fn to_nvim_command(&self, bufnr: i64) -> String {
        match self {
            // TOOD: support build log float as default
            BufferDirection::Default => format!("sbuffer {bufnr}"),
            BufferDirection::Vertical => format!("vert sbuffer {bufnr}"),
            BufferDirection::Horizontal => format!("sbuffer {bufnr}"),
            BufferDirection::TabEdit => format!("tabe {bufnr}"),
        }
    }
}

/// Logging Task
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LoggingTask {
    source: PathBuf,
    status: LoggingTaskStatus,
    title: String,
    description: Option<String>,
}

#[cfg(feature = "neovim")]
impl LuaUserData for LoggingTask {}

/// Logging Task Status
///
/// This is used to indicate the status of a logging task.
#[derive(Clone, Debug, EnumString, EnumDisplay, Serialize, Deserialize)]
pub enum LoggingTaskStatus {
    /// A New Logging task that isn't yet processed
    Created,
    /// A Logging task that is wating for and cosuming loggs
    Consuming,
    /// Like [`LoggingTaskStatus::Cosuming`] but meant for a long running tasks.
    Running,
    /// A Logging task that ended with success stats code
    Succeeded,
    /// A Logging task that errored
    Errored,
}

impl Default for LoggingTaskStatus {
    fn default() -> Self {
        Self::Created
    }
}

#[cfg(feature = "neovim")]
impl<'a> FromLua<'a> for LoggingTaskStatus {
    fn from_lua(value: LuaValue<'a>, _: &'a Lua) -> LuaResult<Self> {
        use std::str::FromStr;
        if let LuaValue::String(ref value) = value {
            Self::from_str(value.to_str()?).to_lua_err()
        } else {
            Err(LuaError::external(
                "Expected a string value for BuildConfiguration",
            ))
        }
    }
}
