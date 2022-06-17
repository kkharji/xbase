mod config;
mod dependency;
mod options;
mod package;
mod platform;
mod target;
mod target_type;

use crate::device::Device;
use crate::util::fs::{
    get_build_cache_dir, get_build_cache_dir_with_config, gitignore_to_glob_patterns,
};
use crate::{error::EnsureOptional, state::State, xcodegen, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::MutexGuard;
use xbase_proto::BuildSettings;

use tap::Pipe;

pub use {
    config::PluginConfig, dependency::*, options::*, package::*, platform::*, target::*,
    target_type::*,
};

/// Represent XcodeGen Project
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    /// Project Name or rather xproj generated file name.
    pub name: String,

    /// The list of targets in the project mapped by name
    pub targets: HashMap<String, ProjectTarget>,

    #[serde(skip)]
    /// Root directory
    pub root: PathBuf,

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

impl Project {
    pub async fn new(root: &std::path::PathBuf) -> Result<Self> {
        let path = xcodegen::config_file(root)?;

        let content = tokio::fs::read_to_string(path).await?;
        let mut project = serde_yaml::from_str::<Project>(&content)?;
        let gitignore_patterns = gitignore_to_glob_patterns(root).await?;

        // Note: Add extra ignore patterns to `ignore` local config requires restarting daemon.
        project.ignore_patterns.extend(gitignore_patterns);
        project.ignore_patterns.extend(vec![
            "**/.git/**".into(),
            "**/*.xcodeproj/**".into(),
            "**/.*".into(),
            "**/build/**".into(),
            "**/buildServer.json".into(),
        ]);

        Ok(project)
    }

    pub async fn update(&mut self) -> Result<()> {
        let Self { root, clients, .. } = self;
        let (clients, root) = (clients.clone(), root.clone());

        *self = Self::new(&root).await?;

        self.root = root;
        self.clients = clients;
        tracing::info!("[Projects] update({:?})", self.name);

        Ok(())
    }

    pub fn get_target(&self, name: &String, platform: Option<Platform>) -> Result<&ProjectTarget> {
        match self.targets.get(name) {
            Some(value) => Ok(value),
            None => match platform {
                Some(platform) => {
                    let key = platform.to_string().pipe(|s| name.replace(&s, ""));
                    self.targets.get(&key).to_result("target", key)
                }
                None => Err(anyhow::anyhow!("No Target found with {name} {:#?}", platform).into()),
            },
        }
    }

    pub async fn remove_target_watchers<'a>(
        &self,
        state: &'a mut MutexGuard<'_, State>,
    ) -> Result<()> {
        state.watcher.get_mut(&self.root)?.listeners.clear();
        Ok(())
    }
}

impl Project {
    /// Generate compile commands for project via compiling all targets
    pub async fn generate_compile_commands(&self) -> Result<()> {
        use xclog::{XCCompilationDatabase, XCCompileCommand};

        tracing::info!("Generating compile commands ... ");
        let mut compile_commands: Vec<XCCompileCommand> = vec![];
        let cache_root = get_build_cache_dir(&self.root)?;
        // Because xcodebuild clean can't remove it
        tokio::fs::remove_dir_all(&cache_root).await?;

        let build_args: Vec<String> = vec![
            "clean".into(),
            "build".into(),
            format!("SYMROOT={cache_root}"),
            "-configuration".into(),
            "Debug".into(),
            "-allowProvisioningUpdates".into(),
        ];
        println!("{build_args:#?}");
        let compile_db = XCCompilationDatabase::generate(&self.root, &build_args).await?;

        compile_db
            .into_iter()
            .for_each(|cmd| compile_commands.push(cmd));

        tracing::info!("Compile Commands Generated");

        let json = serde_json::to_vec_pretty(&compile_commands)?;
        tokio::fs::write(self.root.join(".compile"), &json).await?;

        Ok(())
    }

    pub fn build_args(
        &self,
        build_settings: &BuildSettings,
        device: &Option<Device>,
    ) -> Result<Vec<String>> {
        let mut args = build_settings
            .to_string()
            .split_whitespace()
            .map(ToString::to_string)
            .collect::<Vec<String>>();

        args.remove(0);
        args.insert(0, "build".to_string());

        if let Some(device) = device {
            args.extend(device.special_build_args())
        }

        // TODO: check if scheme isn't already defined!
        args.extend_from_slice(&[
            format!(
                "SYMROOT={}",
                get_build_cache_dir_with_config(&self.root, build_settings)?
            ),
            "-allowProvisioningUpdates".into(),
        ]);

        Ok(args)
    }
}
