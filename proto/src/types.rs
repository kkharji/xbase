use serde::{Deserialize, Serialize};
use std::fmt::Display;
use strum::{Display as EnumDisplay, EnumString};
use xcodeproj::pbxproj::PBXTargetPlatform;

/// Build Configuration to run
#[derive(Clone, Debug, Serialize, Deserialize, EnumDisplay, EnumString)]
pub enum BuildConfiguration {
    Debug,
    Release,
    Custom(String),
}

/// Operation
///
/// Should request be executed once, stoped (if watched) or start new watch service?
#[derive(Clone, Debug, Serialize, Deserialize, EnumDisplay, EnumString)]
pub enum Operation {
    Watch,
    Stop,
    Once,
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

/// Fields required to build a project
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TargetInfo {
    /// Configuration to build with, default Debug
    pub platform: PBXTargetPlatform,
    /// Scheme to build with
    pub watching: bool,
}

#[cfg(feature = "neovim")]
impl<'a> ToLua<'a> for TargetInfo {
    fn to_lua(self, lua: &'a Lua) -> LuaResult<LuaValue<'a>> {
        let table = lua.create_table()?;
        table.set("platform", self.platform.to_string())?;
        table.set("watching", self.watching)?;
        Ok(LuaValue::Table(table))
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

/// Device Lookup information to run built project with
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct DeviceLookup {
    pub name: Option<String>,
    pub id: Option<String>,
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
