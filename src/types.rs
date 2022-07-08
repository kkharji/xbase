use crate::error::*;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use strum::{Display as EnumDisplay, EnumString};
use xcodeproj::pbxproj::PBXTargetPlatform;

pub type Result<T, E = Error> = std::result::Result<T, E>;

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
    #[allow(dead_code)]
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
/// Short hand to get Result with given Error based by anyhow's
#[allow(non_snake_case)]
pub fn OK<T>(t: T) -> Result<T> {
    Result::Ok(t)
}

//pub trait XBase {
//    ///
//    /// Register client and project root.
//    ///
//    /// If the project is already registered then it will not be re-registered and instead
//    /// broadcast address socket will be returned.
//    ///
//    async fn register(root: PathBuf) -> Result<PathBuf>;
//    ///
//    /// Build Project and get path to where to build log will be located
//    ///
//    async fn build(req: BuildRequest) -> Result<()>;
//    ///
//    /// Run Project and get path to where to Runtime log will be located
//    ///
//    async fn run(req: RunRequest) -> Result<()>;
//    ///
//    /// Drop project at a given root
//    ///
//    async fn drop(roots: HashSet<PathBuf>) -> Result<()>;
//    ///
//    /// Return targets for client projects
//    ///
//    async fn targets(root: PathBuf) -> Result<HashMap<String, TargetInfo>>;
//    ///
//    /// Return device names and id for given target
//    ///
//    async fn runners(platform: PBXTargetPlatform) -> Result<Vec<HashMap<String, String>>>;
//    ///
//    /// Return all watched keys
//    ///
//    async fn watching(root: PathBuf) -> Result<Vec<String>>;
//}
