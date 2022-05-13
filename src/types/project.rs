use super::Root;
#[cfg(feature = "daemon")]
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Represent XcodeGen Project
#[derive(Debug, Deserialize, Serialize)]
pub struct Project {
    /// Project Name or rather xproj generated file name.
    pub name: String,
    /// The list of targets in the project mapped by name
    pub targets: HashMap<String, Target>,
    /// XcodeBase local configuration
    #[serde(rename(deserialize = "XcodeBase"), default)]
    pub xcode_base: LocalConfig,
    /// Root directory
    #[serde(skip)]
    pub root: Root,
    /// Connected Clients
    #[serde(default)]
    pub clients: Vec<i32>,
    /// Ignore Patterns
    #[serde(default)]
    pub ignore_patterns: Vec<String>,
}

/// Represent Xcode Target
// TODO: serialize from string or vector
#[derive(Debug, Deserialize, Serialize)]
pub struct Target {
    pub r#type: String,
    pub platform: String,
    pub sources: Vec<PathBuf>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct LocalConfig {
    pub ignore: Vec<String>,
}

#[cfg(feature = "daemon")]
impl Project {
    pub async fn new(root: &PathBuf) -> Result<Self> {
        let path = root.join("project.yml");
        if !path.exists() {
            anyhow::bail!("project.yaml doesn't exist in '{:?}'", root)
        }

        let project = tokio::fs::read_to_string(path).await?;
        let mut project = serde_yaml::from_str::<Project>(&project)?;

        let ignore_patterns: Vec<String> = vec![
            "**/.git/**".into(),
            "**/*.xcodeproj/**".into(),
            "**/.*".into(),
            "**/build/**".into(),
            "**/buildServer.json".into(),
        ];

        // Note: Add extra ignore patterns to `ignore` local config requires restarting daemon.
        project
            .ignore_patterns
            .extend(project.xcode_base.ignore.clone());

        project.ignore_patterns.extend(ignore_patterns);

        Ok(project)
    }

    pub async fn update(&mut self) -> Result<()> {
        let new_project = Self::new(&self.root).await?;
        self.name = new_project.name;
        self.targets = new_project.targets;
        self.xcode_base = new_project.xcode_base;

        Ok(())
    }
}
