use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Represent Xcode Target
#[derive(Debug, Deserialize, Serialize)]
pub struct Target {
    pub r#type: String,
    pub platform: String,
    pub sources: Vec<PathBuf>,
}

pub type TargetMap = HashMap<String, Target>;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct LocalConfig {
    pub ignore: Vec<String>,
}

/// Represent XcodeGen Project
#[derive(Debug, Deserialize, Serialize)]
pub struct Project {
    /// Project Name or rather xproj generated file name.
    name: String,
    /// The list of targets in the project mapped by name
    targets: TargetMap,
    /// XcodeBase local configuration
    #[serde(rename(deserialize = "XcodeBase"), default)]
    xcode_base: LocalConfig,
    /// Root directory
    #[serde(skip)]
    root: PathBuf,
}

impl Project {
    #[cfg(feature = "daemon")]
    pub async fn new(root: &PathBuf) -> anyhow::Result<Self> {
        let path = root.join("project.yml");
        if !path.exists() {
            anyhow::bail!("project.yaml doesn't exist in '{:?}'", root)
        }

        let content = tokio::fs::read_to_string(path).await?;
        if cfg!(feature = "serial") {
            let mut project = serde_yaml::from_str::<Project>(&content)?;
            project.root = root.clone();
            Ok(project)
        } else {
            anyhow::bail!(r#"feature = "serial" is to be created from yml"#)
        }
    }

    pub fn config(&self) -> &LocalConfig {
        &self.xcode_base
    }

    /// Get a reference to the project's name.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Get a reference to the project's targets.
    pub fn targets(&self) -> &TargetMap {
        &self.targets
    }

    /// Get project target from project.targets using target_name
    pub fn get_target(&self, target_name: &str) -> Option<&Target> {
        self.targets().get(target_name)
    }

    pub fn to_string(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string(&self)?)
    }
}
