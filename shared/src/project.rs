use std::collections::HashMap;

use crate::xcode;
use anyhow::{Context, Result};
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
    /// Root directory
    #[serde(skip)]
    root: PathBuf,
}

impl Project {
    pub async fn new_from_project_yml(root: PathBuf, path: PathBuf) -> Result<Self> {
        let content = fs::read_to_string(path).await?;
        let mut project: Project =
            serde_yaml::from_str(&content).context("Unable to parse project.yaml")?;
        project.root = root;
        Ok(project)
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

    /// Build project with clean and return build log
    pub async fn fresh_build(&self) -> Result<Vec<String>> {
        /*
           TODO: Find away to get commands ran without doing xcodebuild clean

           Right now, in order to produce compiled commands and for `xcodebuild build` to spit out ran
           commands, we need to first run xcodebuild clean.

           NOTE: This far from possilbe after some research
        */
        xcode::clean(&self.root, &["-verbose"]).await?;

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
        xcode::build(&self.root, &["-verbose"]).await
    }
}

impl Project {}
