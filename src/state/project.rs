#[cfg(feature = "xcode")]
use crate::xcode;
#[allow(unused_imports)]
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;

/// Represent Xcode Target
#[derive(Debug)]
#[allow(dead_code)]
#[cfg_attr(feature = "serial", derive(serde::Deserialize))]
pub struct Target {
    r#type: String,
    platform: String,
    sources: Vec<PathBuf>,
}

#[derive(Debug)]
#[cfg_attr(feature = "serial", derive(serde::Deserialize))]
pub struct LocalConfig {
    pub ignore: Vec<String>,
}

pub type TargetMap = HashMap<String, Target>;

/// Represent XcodeGen Project
#[derive(Debug)]
#[cfg_attr(feature = "serial", derive(serde::Deserialize))]
pub struct Project {
    /// Project Name or rather xproj generated file name.
    name: String,
    /// The list of targets in the project mapped by name
    targets: TargetMap,
    /// XcodeBase local configuration
    #[cfg_attr(feature = "serial", serde(rename(deserialize = "XcodeBase")))]
    xcode_base: LocalConfig,
    /// Root directory
    #[cfg_attr(feature = "serial", serde(skip))]
    #[allow(dead_code)]
    root: PathBuf,
}

impl Project {
    #[cfg(feature = "async")]
    pub async fn new_from_project_yml(root: PathBuf, path: PathBuf) -> Result<Self> {
        use anyhow::bail;

        let content = tokio::fs::read_to_string(path).await?;
        if cfg!(feature = "serial") {
            let mut project = serde_yaml::from_str::<Project>(&content)?;
            project.root = root;
            Ok(project)
        } else {
            bail!(r#"feature = "serial" is to be created from yml"#)
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

    /// Build project with clean and return build log
    #[cfg(all(feature = "async", feature = "xcode"))]
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
