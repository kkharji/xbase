mod barebone;
mod swift;
mod tuist;
mod xcodegen;

use crate::{device::*, run::*, util::*, watch::*};
use crate::{Result, StringStream};
use anyhow::Context;
use async_stream::stream;
use barebone::BareboneProject;
use futures::StreamExt;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use xbase_proto::{BuildSettings, Client};
use xclog::{XCBuildSettings, XCLogger};
use xcodeproj::pbxproj::PBXTargetPlatform;
use {swift::*, tuist::*, xcodegen::*};

/// Project Data
pub trait ProjectData: std::fmt::Debug {
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
    ) -> Result<(StringStream, Vec<String>)> {
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

        log::trace!("building with [{}]", args.join(" "));

        let mut xclogger = XCLogger::new(self.root(), &args)?;
        let stream = stream! {
            while let Some(output) =  xclogger.next().await {
                if output.is_result() && output.starts_with("[Exit]") {
                    if !output.strip_prefix("[Exit] ").map(|s| s == "0").unwrap_or_default() {
                        yield String::from("FAILED")
                    }
                } else {
                    yield output.to_string()
                }
            }
        };

        Ok((stream.boxed(), args))
    }

    /// Get build cache root
    fn build_cache_root(&self) -> Result<String> {
        let get_build_cache_dir = fs::get_build_cache_dir(self.root())?;
        std::fs::remove_dir_all(&get_build_cache_dir).ok();
        Ok(get_build_cache_dir)
    }
}

#[async_trait::async_trait]
pub trait ProjectRun: ProjectData + ProjectBuild {
    fn get_runner(
        &self,
        cfg: &BuildSettings,
        device: Option<&Device>,
    ) -> Result<(Box<dyn Runner + Send + Sync>, StringStream)> {
        log::info!("Running {}", self.name());

        let (build_stream, args) = self.build(cfg, device)?;
        let info = XCBuildSettings::new_sync(self.root(), &args)?;
        let runner: Box<dyn Runner + Send + Sync> = match device {
            Some(device) => Box::new(SimulatorRunner::new(device.clone(), &info)),
            None => Box::new(BinRunner::from_build_info(&info)),
        };

        Ok((runner, build_stream))
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
    + ProjectRun
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
    } else if root.join("Package.swift").exists() {
        Ok(Box::new(SwiftProject::new(client).await?))
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
        "**/.build/**".into(),
        "**/buildServer.json".into(),
        "**/DerivedData/**".into(),
        "**/Derived/**".into(),
    ];

    default.extend(
        fs::gitignore_to_glob_patterns(root)
            .await
            .unwrap_or_default(),
    );

    default.dedup();

    default
}
