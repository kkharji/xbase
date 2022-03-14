use std::collections::HashMap;

use anyhow::Context;
use serde::Deserialize;
use std::path::PathBuf;
use tokio::fs;

/// Represent Xcode Target
#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct Target {
    r#type: String,
    platform: String,
    sources: Vec<PathBuf>,
}
#[derive(Deserialize, Debug)]
pub struct LocalConfig {
    pub ignore: Vec<String>,
}

pub type TargetMap = HashMap<String, Target>;

/// Represent XcodeGen Project
#[derive(Deserialize, Debug)]
pub struct Project {
    /// Project Name or rather xproj generated file name.
    name: String,
    /// The list of targets in the project mapped by name
    targets: TargetMap,
    /// XcodeBase local configuration
    #[serde(rename(deserialize = "XcodeBase"))]
    xcode_base: LocalConfig,
}

impl Project {
    pub async fn new_from_project_yml(path: PathBuf) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path).await?;
        serde_yaml::from_str(&content).context("Unable to parse project.yaml")
    }

    pub fn config(&self) -> &LocalConfig {
        &self.xcode_base
    }
}

impl Project {
    /// Get a reference to the project's name.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Get a reference to the project's targets.
    pub fn targets(&self) -> &TargetMap {
        &self.targets
    }
}
