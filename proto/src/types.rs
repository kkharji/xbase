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
#[derive(Clone, Debug, Serialize, Deserialize, EnumDisplay, EnumString)]
#[repr(u8)]
pub enum Operation {
    /// Execute the requested operation once
    Once,
    /// Start a watch service the requested operation
    WatchStart,
    /// Start existing service the requested operation
    WatchStop,
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
#[repr(C)]
pub struct TargetInfo {
    /// Configuration to build with, default Debug
    pub platform: PBXTargetPlatform,
    /// Scheme to build with
    pub watching: bool,
}

/// Device Lookup information to run built project with
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct DeviceLookup {
    pub name: Option<String>,
    pub id: Option<String>,
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
    /// Returns `true` if the request kind is [`WatchStart`].
    ///
    /// [`WatchStart`]: RequestKind::WatchStart
    #[must_use]
    pub fn is_watch(&self) -> bool {
        matches!(self, Self::WatchStart)
    }

    /// Returns `true` if the request kind is [`WatchStop`].
    ///
    /// [`WatchStop`]: RequestKind::WatchStop
    #[must_use]
    pub fn is_stop(&self) -> bool {
        matches!(self, Self::WatchStop)
    }

    /// Returns `true` if the request kind is [`Once`].
    ///
    /// [`Once`]: RequestKind::Once
    #[must_use]
    pub fn is_once(&self) -> bool {
        matches!(self, Self::Once)
    }
}
