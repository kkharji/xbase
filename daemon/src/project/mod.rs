mod barebone;
mod swift;
mod tuist;
mod xcodegen;

use crate::broadcast::{self, Broadcast};
use crate::Result;
use crate::{device::*, run::*, util::*, watch::*};
use anyhow::Context;
use barebone::BareboneProject;
use process_stream::ProcessExt;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use xbase_proto::BuildSettings;
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
    fn clients(&self) -> &i32;
    /// Get mut clients
    fn clients_mut(&mut self) -> &mut i32;
    /// Increment the number of client connected to project
    fn inc_clients(&mut self) {
        let current = self.clients_mut();
        *current += 1;
    }
    /// Decrement the number of client connected to project
    fn dec_clients(&mut self) {
        let current = self.clients_mut();
        *current -= 1;
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
        broadcast: &Arc<Broadcast>,
    ) -> Result<Vec<String>> {
        let mut args = cfg.to_args();
        let target = &cfg.target;
        let name = self.name().to_owned();
        let xcworkspace = format!("{}.xcworkspace", &name);

        args.insert(0, "build".to_string());

        broadcast::notify_info!(broadcast, "[target: {target}] building ...")?;

        if let Some(device) = device {
            args.extend(device.special_build_args())
        }

        let cache_build_root = fs::get_build_cache_dir_with_config(self.root(), cfg)?;

        args.push(format!("SYMROOT={cache_build_root}",));
        args.push("-allowProvisioningUpdates".into());

        if self.root().join(&xcworkspace).exists() {
            args.iter_mut().for_each(|arg| {
                if arg == "-target" {
                    *arg = "-scheme".into()
                }
            });
            args.extend_from_slice(&["-workspace".into(), xcworkspace]);
        } else {
            args.extend_from_slice(&["-project".into(), format!("{}.xcodeproj", name)]);
        }

        broadcast::log_trace!(broadcast, "building with [{}]", args.join(" "))?;

        let success = broadcast
            .consume(Box::new(XCLogger::new(self.root(), &args)?))?
            .blocking_recv()
            .unwrap_or_default();

        if !success {
            broadcast::notify_error!(
                broadcast,
                "[target: {target}] building Failed: `xcodebuild {}`",
                args.join(" ")
            )?
        } else {
            broadcast::notify_error!(broadcast, "[target: {target}] build failed")?;
        };

        Ok(args)
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
        broadcast: &Arc<Broadcast>,
    ) -> Result<(Box<dyn Runner + Send + Sync>, Vec<String>)> {
        let args = self.build(cfg, device, broadcast)?;
        let info = XCBuildSettings::new_sync(self.root(), &args)?;
        let runner: Box<dyn Runner + Send + Sync> = match device {
            Some(device) => Box::new(SimulatorRunner::new(device.clone(), &info)),
            None => Box::new(BinRunner::from_build_info(&info)),
        };

        Ok((runner, args))
    }
}

#[async_trait::async_trait]
pub trait ProjectCompile: ProjectData {
    /// Generate compile database in project root
    async fn update_compile_database(&self, broadcast: &Arc<Broadcast>) -> Result<()>;

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
    async fn generate(&mut self, broadcast: &Arc<Broadcast>) -> Result<()>;
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
    async fn new(root: &PathBuf, broadcast: &Arc<Broadcast>) -> Result<Self>
    where
        Self: Sized;
}

erased_serde::serialize_trait_object!(Project);

/// Create a project from given client
pub async fn project(
    root: &PathBuf,
    broadcast: &Arc<Broadcast>,
) -> Result<Box<dyn Project + Send + Sync>> {
    if root.join("project.yml").exists() {
        Ok(Box::new(XCodeGenProject::new(root, broadcast).await?))
    } else if root.join("Project.swift").exists() {
        Ok(Box::new(TuistProject::new(root, broadcast).await?))
    } else if root.join("Package.swift").exists() {
        Ok(Box::new(SwiftProject::new(root, broadcast).await?))
    } else {
        Ok(Box::new(BareboneProject::new(root, broadcast).await?))
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
