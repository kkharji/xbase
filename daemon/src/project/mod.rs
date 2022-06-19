mod generator;
mod platform;

use crate::device::Device;
use crate::util::fs::{
    get_build_cache_dir, get_build_cache_dir_with_config, gitignore_to_glob_patterns,
};
use crate::Error;
use crate::{state::State, Result};
use anyhow::Context;
use generator::ProjectGenerator;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::MutexGuard;
use xbase_proto::BuildSettings;
use xcodeproj::XCodeProject;

pub use platform::*;

/// Project Inner
#[derive(Debug)]
pub enum ProjectInner {
    None,
    XCodeProject(XCodeProject),
    Swift,
}

impl Default for ProjectInner {
    fn default() -> Self {
        Self::None
    }
}

/// Represent XcodeGen Project
#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    /// Project Name or rather xproj generated file name.
    pub name: String,

    /// The list of targets in the project mapped by name
    pub targets: HashMap<String, Platform>,

    /// Root directory
    #[serde(skip)]
    pub root: PathBuf,

    /// Connected Clients
    #[serde(default)]
    pub clients: Vec<i32>,

    /// Ignore Patterns
    #[serde(default)]
    pub ignore_patterns: Vec<String>,

    /// Generator
    pub generator: ProjectGenerator,

    #[serde(skip)]
    /// XCodeProject Data
    inner: ProjectInner,
}

impl Project {
    pub async fn new(root: &std::path::PathBuf) -> Result<Self> {
        let mut project = Self::default();

        project.generator = ProjectGenerator::new(root);

        project.inner = if root.join("Package.swift").exists() {
            tracing::debug!("[Project] Kind = Swift",);
            project.name = "UnknownSwiftProject".into();
            ProjectInner::Swift
        } else {
            tracing::debug!("[Project] Kind = XCodeProject");
            tracing::debug!("[Project] Generator = {:?}", project.generator);
            let matches = wax::walk("*.xcodeproj", root, 1)
                .context("Glob")?
                .flatten()
                .map(|entry| entry.into_path())
                .collect::<Vec<PathBuf>>();

            let path = if matches.is_empty() {
                return Err(Error::Register("Expected swift or xcode project".into()));
            } else {
                &matches[0]
            };

            let xcodeproj = XCodeProject::new(path)?;

            project.name = path
                .file_name()
                .and_then(|name| Some(name.to_str()?.split_once(".")?.0.to_string()))
                .unwrap();

            tracing::debug!("[Project] name = {}", project.name);
            project.targets = xcodeproj
                .targets()
                .into_iter()
                .flat_map(|t| {
                    let name = t.name?.to_string();
                    let sdkroots = t.sdkroots();
                    let sdkroot = sdkroots.first()?;
                    let platform = Platform::from_sdk_root(sdkroot.as_str());
                    Some((name, platform))
                })
                .collect::<HashMap<_, _>>();
            tracing::debug!("[Project] targets = {:?}", project.targets);
            ProjectInner::XCodeProject(xcodeproj)
        };

        project
            .ignore_patterns
            .extend(gitignore_to_glob_patterns(root).await?);

        project.ignore_patterns.extend(vec![
            "**/.git/**".into(),
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
