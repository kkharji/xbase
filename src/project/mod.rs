mod barebone;
mod swift;
mod tuist;
mod xcodegen;

use crate::util::PathExt;
use crate::*;
use anyhow::Context;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use xclog::{XCBuildSettings, XCLogger};

/// Project Data
pub trait ProjectData: std::fmt::Debug {
    /// Project root
    fn root(&self) -> &PathBuf;
    /// Project name
    fn name(&self) -> &str;
    /// Project targets
    fn targets(&self) -> &HashMap<String, TargetInfo>;
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
    ) -> Result<(Vec<String>, tokio::sync::mpsc::Receiver<bool>)> {
        let mut args = cfg.to_args();
        let target = &cfg.target;
        let name = self.name().to_owned();
        let xcworkspace = format!("{}.xcworkspace", &name);
        let task = Task::new(TaskKind::Build, target, broadcast.clone());

        args.insert(0, "build".to_string());

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

        task.debug(format!("[{target}] {}", args.join(" ")));

        let recv = task.consume(Box::new(XCLogger::new(self.root(), &args)?))?;

        Ok((args, recv))
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
    ) -> Result<(
        Box<dyn Runner + Send + Sync>,
        Vec<String>,
        tokio::sync::mpsc::Receiver<bool>,
    )> {
        let (args, recv) = self.build(cfg, device, broadcast)?;

        let info = XCBuildSettings::new_sync(self.root(), &args)?;

        let runner: Box<dyn Runner + Send + Sync> = match device {
            Some(device) => Box::new(SimulatorRunner::new(device.clone(), &info)),
            None => Box::new(BinRunner::from_build_info(&info)),
        };

        Ok((runner, args, recv))
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
    ProjectData + ProjectBuild + ProjectRun + ProjectCompile + ProjectGenerate + Sync + Send
{
    /// Create new project
    async fn new(root: &PathBuf, broadcast: &Arc<Broadcast>) -> Result<Self>
    where
        Self: Sized;

    /// Ensure Daemon support for given project
    async fn ensure_setup(
        &mut self,
        event: Option<&Event>,
        broadcast: &Arc<Broadcast>,
    ) -> Result<bool> {
        use tokio::fs::File;
        use tokio::io::AsyncWriteExt;

        let root = self.root();
        let compile_path = root.join(".compile");
        let is_swift_project = root.join("Package.swift").exists();

        /// Server Config
        static BUILD_SERVER_CONFIG: Lazy<Vec<u8>> = Lazy::new(|| {
            let path: PathBuf = BIN_ROOT
                .replace("$HOME", &std::env::var("HOME").unwrap())
                .into();
            serde_json::json!({
                "name": "XBase",
                "argv": [path.join("xbase-sourcekit-helper")],
                "version": "0.3",
                "bspVersion": "0.2",
                "languages": ["swift", "objective-c", "objective-cpp", "c", "cpp"]
            })
            .to_string()
            .into_bytes()
        });

        if !is_swift_project {
            let build_server_path = root.join("buildServer.json");
            let build_server_file_exists = build_server_path.exists();

            let outdated = if build_server_file_exists {
                let content = tokio::fs::read(&build_server_path).await?;
                serde_json::from_slice::<serde_json::Value>(&content)
                    .unwrap()
                    .get("version")
                    .and_then(|v| v.as_str())
                    .map(|v| v != "0.3")
                    .unwrap_or_default()
            } else {
                false
            };

            if !build_server_path.exists() || outdated {
                // NOTE: Use broadcast
                tracing::info!("Creating {:?}", build_server_path);
                let mut build_server_file = File::create(build_server_path).await?;
                build_server_file.write_all(&BUILD_SERVER_CONFIG).await?;
                build_server_file.sync_all().await?;
                build_server_file.shutdown().await?;
            }
        };

        if let Some(event) = event {
            if self.should_generate(event) {
                self.generate(broadcast).await?;
                self.update_compile_database(broadcast).await?;
                return Ok(true);
            }
        }

        if !is_swift_project && !compile_path.exists() {
            self.update_compile_database(broadcast).await.ok();
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

/// Alias for Box Project
pub type ProjectImpl = Box<dyn Project + Send + Sync>;

/// Create a project from given client
pub async fn project(root: &PathBuf, broadcast: &Arc<Broadcast>) -> Result<ProjectImpl> {
    Ok(if root.join("project.yml").exists() {
        Box::new(xcodegen::XCodeGenProject::new(root, broadcast).await?)
    } else if root.join("Project.swift").exists() {
        Box::new(tuist::TuistProject::new(root, broadcast).await?)
    } else if root.join("Package.swift").exists() {
        Box::new(swift::SwiftProject::new(root, broadcast).await?)
    } else {
        Box::new(barebone::BareboneProject::new(root, broadcast).await?)
    })
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
