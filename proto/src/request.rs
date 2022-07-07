use crate::types::*;
use crate::util::value_or_default;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, path::PathBuf};

/// Request to build a particular project
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuildRequest {
    pub root: PathBuf,
    pub settings: BuildSettings,
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
    pub ops: Operation,
}

impl Display for BuildRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:Build:{}", self.root.display(), self.settings)
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
