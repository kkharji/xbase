use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use strum::{Display as EnumDisplay, EnumString};

/// Client data
#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct Client {
    pub pid: i32,
    pub root: PathBuf,
    pub address: String,
}

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

/// Log Buffer open direction
#[derive(Clone, Debug, strum::EnumString, Serialize, Deserialize)]
#[strum(ascii_case_insensitive)]
pub enum BufferDirection {
    Default,
    Vertical,
    Horizontal,
    TabEdit,
}

/// Device Lookup information to run built project with
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct DeviceLookup {
    pub name: Option<String>,
    pub udid: Option<String>,
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
