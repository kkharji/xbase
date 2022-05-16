mod config;
mod dependency;
mod options;
mod package;
mod platform;
mod target;
mod target_type;

use super::Root;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(any(feature = "daemon", feature = "lua"))]
use tap::Pipe;

pub use {
    config::PluginConfig, dependency::*, options::*, package::*, platform::*, target::*,
    target_type::*,
};

#[cfg(feature = "daemon")]
use anyhow::Result;

/// Represent XcodeGen Project
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    /// Project Name or rather xproj generated file name.
    pub name: String,

    /// The list of targets in the project mapped by name
    pub targets: HashMap<String, ProjectTarget>,

    #[serde(rename(deserialize = "xbase"), default)]
    /// xbase local configuration
    pub xbase: PluginConfig,

    #[serde(skip)]
    /// Root directory
    pub root: Root,

    #[serde(default)]
    /// Connected Clients
    pub clients: Vec<i32>,

    #[serde(default)]
    /// Ignore Patterns
    pub ignore_patterns: Vec<String>,

    #[serde(default)]
    /// Options to override default behaviour
    pub options: ProjectOptions,

    #[serde(default)]
    /// Packages
    pub packages: HashMap<String, ProjectPackage>,
}

#[cfg(feature = "daemon")]
impl Project {
    pub async fn new(root: &std::path::PathBuf) -> Result<Self> {
        let path = root.join("project.yml");
        if !path.exists() {
            anyhow::bail!("project.yaml doesn't exist in '{:?}'", root)
        }

        let content = tokio::fs::read_to_string(path).await?;
        let mut project = serde_yaml::from_str::<Project>(&content)?;

        // Note: Add extra ignore patterns to `ignore` local config requires restarting daemon.
        project.ignore_patterns.extend(project.xbase.ignore.clone());
        project
            .ignore_patterns
            .extend(DEFAULT_IGNORE_PATTERN.clone().into_iter());

        Ok(project)
    }

    pub async fn update(&mut self) -> Result<()> {
        let ignore_patterns = self.ignore_patterns.clone();
        *self = Self::new(&self.root).await?;
        self.ignore_patterns = ignore_patterns;
        Ok(())
    }

    pub fn get_target(&self, name: &String, platform: Option<Platform>) -> Result<&ProjectTarget> {
        match self.targets.get(name) {
            Some(value) => Ok(value),
            None => match platform {
                Some(platform) => {
                    let key = match platform {
                        Platform::IOS => "iOS",
                        Platform::WatchOS => "watchOS",
                        Platform::TvOS => "tvOS",
                        Platform::MacOS => "macOS",
                        Platform::None => "",
                    }
                    .pipe(|s| name.replace(s, ""));

                    self.targets
                        .get(&key)
                        .ok_or_else(|| anyhow::anyhow!("No target found for {key}"))
                }
                None => anyhow::bail!("No Target found with {name} {:#?}", platform),
            },
        }
    }
}
#[cfg(feature = "daemon")]
lazy_static::lazy_static! {
    static ref DEFAULT_IGNORE_PATTERN: Vec<String> = vec![
            "**/.git/**".into(),
            "**/*.xcodeproj/**".into(),
            "**/.*".into(),
            "**/build/**".into(),
            "**/buildServer.json".into(),
        ];
}
