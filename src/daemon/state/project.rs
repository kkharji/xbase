#[cfg(feature = "daemon")]
use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::path::PathBuf;

/// Represent Xcode Target
#[derive(Debug)]
#[allow(dead_code)]
#[cfg_attr(feature = "serial", derive(Deserialize, Serialize))]
pub struct Target {
    r#type: String,
    platform: String,
    sources: Vec<PathBuf>,
}

#[derive(Debug, Default)]
#[cfg_attr(feature = "serial", derive(Deserialize, Serialize))]
pub struct LocalConfig {
    pub ignore: Vec<String>,
}

pub type TargetMap = HashMap<String, Target>;

/// Represent XcodeGen Project
#[derive(Debug)]
#[cfg_attr(feature = "serial", derive(Deserialize, Serialize))]
pub struct Project {
    /// Project Name or rather xproj generated file name.
    name: String,
    /// The list of targets in the project mapped by name
    targets: TargetMap,
    /// XcodeBase local configuration
    #[cfg_attr(feature = "serial", serde(rename(deserialize = "XcodeBase"), default))]
    xcode_base: LocalConfig,
    /// Root directory
    #[cfg_attr(feature = "serial", serde(skip))]
    #[allow(dead_code)]
    root: PathBuf,
}

impl Project {
    #[cfg(feature = "async")]
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

    /// Build project with clean and return build log
    #[cfg(feature = "async")]
    pub async fn fresh_build<'a>(
        &'a self,
    ) -> anyhow::Result<impl tokio_stream::Stream<Item = xcodebuild::parser::Step> + 'a> {
        /*
           TODO: Find away to get commands ran without doing xcodebuild clean

           Right now, in order to produce compiled commands and for `xcodebuild build` to spit out ran
           commands, we need to first run xcodebuild clean.

           NOTE: This far from possilbe after some research
        */
        xcodebuild::runner::spawn_once(&self.root, &["clean"]).await?;

        /*
           TODO: Support overriding xcodebuild arguments

           Not sure how important is this, but ideally I'd like to be able to add extra arguments for
           when generating compiled commands, as well as doing actual builds and runs.

           ```yaml
           XcodeBase:
           buildArguments: [];
           compileArguments: [];
           runArguments: [];
           ```
        */

        xcodebuild::runner::spawn(&self.root, &["build"]).await
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

    #[cfg(feature = "daemon")]
    pub fn nvim_update_state_script(&self) -> anyhow::Result<String> {
        Ok(format!(
            "require'xcodebase.state'.projects['{}'] = vim.json.decode([[{}]])",
            self.root.display(),
            serde_json::to_string(&self)?
        ))
    }
}
