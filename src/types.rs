use crate::error::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display};
use strum::{Display as EnumDisplay, EnumString};
use typescript_type_def::TypeDef;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Serialize, TypeDef)]
pub struct ProjectInfo {
    /// Get watched configurations for given root
    pub watchlist: Vec<String>,
    /// Get targets information for a registers project with a given root
    pub targets: HashMap<String, TargetInfo>,
}

/// Type of operation for building/ruuning a target/scheme
#[derive(Clone, Debug, Serialize, Deserialize, EnumDisplay, EnumString, TypeDef)]
pub enum Operation {
    Watch,
    Stop,
    Once,
}

/// Build Settings used in building/running a target/scheme
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TypeDef)]
pub struct BuildSettings {
    /// Target to build
    pub target: String,
    /// Configuration to build with, default Debug
    pub configuration: String,
    /// Scheme to build with
    pub scheme: Option<String>,
}

/// Target specfic information
#[derive(Clone, Debug, Serialize, Deserialize, TypeDef)]
pub struct TargetInfo {
    pub platform: String,
}

/// Device Lookup information to run built project with
#[derive(Clone, Default, Debug, Serialize, Deserialize, TypeDef)]
pub struct DeviceLookup {
    pub name: String,
    pub id: String,
}

impl DeviceLookup {
    pub fn new(name: String, id: String) -> Self {
        Self { name, id }
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
