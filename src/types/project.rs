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
use {
    crate::{error::EnsureOptional, state::State, xcodegen, Result},
    tokio::sync::MutexGuard,
};

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
        let path = xcodegen::config_file(root)?;

        let content = tokio::fs::read_to_string(path).await?;
        let mut project = serde_yaml::from_str::<Project>(&content)?;

        // Note: Add extra ignore patterns to `ignore` local config requires restarting daemon.
        project.ignore_patterns.extend(project.xbase.ignore.clone());
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

#[cfg(feature = "daemon")]
impl Project {
    /// Generate compile commands for project
    pub async fn generate_compile_commands(&self) -> Result<()> {
        use crate::xcode::append_build_root;
        use crate::CompileError;
        use xclog::XCCompilationDatabase;

        let args = append_build_root(&self.root, None, vec!["clean".into(), "build".into()])?;
        let compile_commands = XCCompilationDatabase::generate(&self.root, &args).await?;

        if compile_commands.is_empty() {
            return Err(CompileError::Empty(self.root.clone()).into());
        }

        compile_commands.write(self.root.join(".compile")).await?;

        Ok(())
    }
}
