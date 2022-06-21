mod barebone;
mod tuist;
mod xcodegen;

use barebone::BareboneProject;
use tuist::TuistProject;
use xclog::XCLogger;
use xcodegen::XCodeGenProject;

use crate::device::Device;
use crate::util::consume_and_log;
use crate::util::fs;
use crate::watch::Event;
use crate::Result;
use anyhow::Context;
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use xbase_proto::{BuildSettings, Client};
use xcodeproj::pbxproj::PBXTargetPlatform;

/// Project Data
pub trait ProjectData: Debug {
    /// Project root
    fn root(&self) -> &PathBuf;
    /// Project name
    fn name(&self) -> &str;
    /// Project targets
    fn targets(&self) -> &HashMap<String, PBXTargetPlatform>;
    /// Project clients
    fn clients(&self) -> &Vec<i32>;
    /// Get mut clients
    fn clients_mut(&mut self) -> &mut Vec<i32>;
    /// Add client to project
    fn add_client(&mut self, pid: i32) {
        self.clients_mut().push(pid)
    }
    /// Get Ignore patterns
    fn watchignore(&self) -> &Vec<String>;
    /// read dir and get xcodeproj paths
    fn get_xcodeproj_paths(&self) -> Result<Vec<PathBuf>> {
        Ok(wax::walk("*.xcodeproj", &self.root())
            .context("Glob")?
            .flatten()
            .map(|entry| entry.into_path())
            .collect::<Vec<PathBuf>>())
    }
}

#[async_trait::async_trait]
pub trait ProjectBuild: ProjectData {
    /// Build Project using BuildSettings and optionally a device
    fn build(
        &self,
        cfg: &BuildSettings,
        device: Option<&Device>,
    ) -> Result<(XCLogger, Vec<String>)> {
        let args = self.build_arguments(&cfg, device)?;
        let xclogger = XCLogger::new(self.root(), &args)?;
        Ok((xclogger, args))
    }
    /// Get build cache root
    fn build_cache_root(&self) -> Result<String> {
        let get_build_cache_dir = fs::get_build_cache_dir(self.root())?;
        std::fs::remove_dir_all(&get_build_cache_dir).ok();
        Ok(get_build_cache_dir)
    }

    /// Build project with given build settings
    fn build_arguments(&self, cfg: &BuildSettings, device: Option<&Device>) -> Result<Vec<String>> {
        let mut args = cfg.to_args();

        args.insert(0, "build".to_string());

        if let Some(device) = device {
            args.extend(device.special_build_args())
        }

        let cache_build_root = fs::get_build_cache_dir_with_config(self.root(), cfg)?;
        // TODO: check if scheme isn't already defined!
        args.extend_from_slice(&[
            format!("SYMROOT={cache_build_root}",),
            "-allowProvisioningUpdates".into(),
            "-project".into(),
            format!("{}.xcodeproj", self.name()),
        ]);

        Ok(args)
    }
}

#[async_trait::async_trait]
pub trait ProjectCompile: ProjectData {
    /// Generate compile database in project root
    async fn update_compile_database(&self) -> Result<()>;

    /// Get compile arguments
    fn compile_arguments(&self) -> Vec<String> {
        vec![
            "clean",
            "build",
            "-configuration",
            "Debug",
            "CODE_SIGN_IDENTITY=\"\"",
            "CODE_SIGNING_REQUIRED=\"NO\"",
            "CODE_SIGN_ENTITLEMENTS=\"\"",
            "CODE_SIGNING_ALLOWED=\"NO\"",
        ]
        .iter()
        .map(ToString::to_string)
        .collect()
    }
}

#[async_trait::async_trait]
pub trait ProjectGenerate: ProjectData {
    /// Whether the project should be generated
    fn should_generate(&self, _event: &Event) -> bool {
        false
    }
    /// Generate xcodeproj
    async fn generate(&mut self) -> Result<()>;
}

#[async_trait::async_trait]
/// Project Extension that can be built, ran and regenerated
pub trait Project:
    ProjectData
    + ProjectBuild
    + ProjectCompile
    + ProjectGenerate
    + Sync
    + Send
    + erased_serde::Serialize
{
    /// Create new project
    async fn new(client: &Client) -> Result<Self>
    where
        Self: Sized;
}

erased_serde::serialize_trait_object!(Project);

/// Create a project from given client
pub async fn project(client: &Client) -> Result<Box<dyn Project + Send + Sync>> {
    let Client { root, .. } = client;
    if root.join("project.yml").exists() {
        Ok(Box::new(XCodeGenProject::new(client).await?))
    } else if root.join("Project.swift").exists() {
        Ok(Box::new(TuistProject::new(client).await?))
    } else {
        Ok(Box::new(BareboneProject::new(client).await?))
    }
}

async fn generate_watchignore<P: AsRef<Path>>(root: P) -> Vec<String> {
    let mut default = vec![
        "**/.git/**".into(),
        "**/.*".into(),
        "**/.compile".into(),
        "**/build/**".into(),
        "**/buildServer.json".into(),
        "**/DerivedData/**".into(),
        "**/Derived/**".into(),
    ];

    default.extend(
        fs::gitignore_to_glob_patterns(root)
            .await
            .unwrap_or_default(),
    );
    default
}
