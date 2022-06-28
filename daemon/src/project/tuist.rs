use super::*;
use crate::util::fs::which;
use crate::watch::Event;
use crate::{Error, Result};
use process_stream::Process;
use serde::Serialize;
use std::{collections::HashMap, path::PathBuf};
use xcodeproj::{pbxproj::PBXTargetPlatform, XCodeProject};

#[derive(Debug, Serialize, Default)]
#[serde(default)]
pub struct TuistProject {
    root: PathBuf,
    targets: HashMap<String, PBXTargetPlatform>,
    num_clients: i32,
    watchignore: Vec<String>,
    #[serde(skip)]
    xcodeproj: XCodeProject,
    #[serde(skip)]
    xcodeproj_path: PathBuf,
    #[serde(skip)]
    manifest: XCodeProject,
    #[serde(skip)]
    manifest_path: PathBuf,
    #[serde(skip)]
    manifest_files: Vec<String>,
}

impl ProjectData for TuistProject {
    fn root(&self) -> &PathBuf {
        &self.root
    }

    fn name(&self) -> &str {
        &self.xcodeproj.name()
    }

    fn targets(&self) -> &HashMap<String, PBXTargetPlatform> {
        &self.targets
    }

    fn clients(&self) -> &i32 {
        &self.num_clients
    }

    fn clients_mut(&mut self) -> &mut i32 {
        &mut self.num_clients
    }

    fn watchignore(&self) -> &Vec<String> {
        &self.watchignore
    }
}

#[async_trait::async_trait]
impl ProjectBuild for TuistProject {}

#[async_trait::async_trait]
impl ProjectRun for TuistProject {}

#[async_trait::async_trait]
impl ProjectCompile for TuistProject {
    // TODO: Use logger
    async fn update_compile_database(&self, logger: &Arc<Broadcast>) -> Result<()> {
        use xclog::XCCompileCommand as C;

        let name = self.name();
        let root = self.root();
        let cache_root = self.build_cache_root()?;
        let arguments = self.compile_arguments();
        let mut compile_commands: Vec<C> = vec![];

        // Compile manifests
        let (mut manifest_compile_success, manifest_compile_commands) = {
            let mut arguments = arguments.clone();

            arguments.extend_from_slice(&[
                format!("SYMROOT={cache_root}_tuist"),
                "-workspace".into(),
                "Manifests.xcworkspace".into(),
                "-scheme".into(),
                "Manifests".into(),
            ]);

            log::debug!("\n\nxcodebuild {}\n", arguments.join(" "));

            let xclogger = XCLogger::new(&root, &arguments)?;
            let xccommands = xclogger.compile_commands.clone();
            let recv = logger.consume(Box::new(xclogger))?;
            (recv, xccommands)
        };

        // Compile Project
        let (mut project_compile_success, project_compile_commands) = {
            let mut arguments = arguments.clone();

            arguments.extend_from_slice(&[
                format!("SYMROOT={cache_root}"),
                "-workspace".into(),
                format!("{name}.xcworkspace"),
                "-scheme".into(),
                format!("{name}"),
            ]);

            log::debug!("\n\nxcodebuild {}\n", arguments.join(" "));

            let xclogger = XCLogger::new(&root, &arguments)?;
            let xccommands = xclogger.compile_commands.clone();
            let recv = logger.consume(Box::new(xclogger))?;
            (recv, xccommands)
        };

        if !(project_compile_success.recv().await.unwrap_or_default()
            && manifest_compile_success.recv().await.unwrap_or_default())
        {
            logger.error(format!(
                "Fail to generated compile commands for {}",
                self.name()
            ))?;
            return Err(Error::Build(self.name().into()));
        }

        compile_commands.extend(manifest_compile_commands.lock().await.to_vec());
        compile_commands.extend(project_compile_commands.lock().await.to_vec());

        log::debug!("[{}] compiled successfully", self.name());
        let json = serde_json::to_vec_pretty(&compile_commands)?;
        tokio::fs::write(root.join(".compile"), &json).await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl ProjectGenerate for TuistProject {
    fn should_generate(&self, event: &Event) -> bool {
        log::trace!("manifest files {:?}", self.manifest_files);
        let is_config_file = self.manifest_files.contains(event.file_name());
        let is_content_update = event.is_content_update_event();
        let is_config_file_update = is_content_update && is_config_file;

        is_config_file_update
            || event.is_create_event()
            || event.is_remove_event()
            || event.is_rename_event()
    }

    /// Generate xcodeproj
    async fn generate(&mut self, _logger: &Arc<Broadcast>) -> Result<()> {
        log::info!("generating ...");

        self.tuist(&["edit", "--permanent"]).await?;
        self.tuist(&["generate", "--no-open"]).await?;

        let (xcodeproj_path, manifest_path) = self.xcodeproj_paths()?;
        let (xcodeproj_path, manifest_path) = (xcodeproj_path.unwrap(), manifest_path.unwrap());

        self.manifest = XCodeProject::new(&manifest_path)?;
        self.manifest_path = manifest_path;
        self.xcodeproj = XCodeProject::new(&xcodeproj_path)?;
        self.xcodeproj_path = xcodeproj_path;
        self.targets = self.xcodeproj.targets_platform();

        Ok(())
    }
}

impl TuistProject {
    pub fn xcodeproj_paths(&self) -> Result<(Option<PathBuf>, Option<PathBuf>)> {
        let paths = self.get_xcodeproj_paths()?;
        if paths.is_empty() {
            return Ok((None, None));
        }

        let (mut xcodeproj, mut manifest) = (None, None);
        if paths.len() > 2 {
            log::warn!(
                "Expected `2` xcodeproj Manifest and Main but found `{}`",
                paths.len()
            )
        }

        paths.into_iter().for_each(|p| {
            if p.file_name()
                .and_then(|s| s.to_str())
                .map(|s| s.starts_with("Manifests"))
                .unwrap_or_default()
            {
                manifest = p.into();
            } else {
                xcodeproj = p.into();
            }
        });
        Ok((xcodeproj, manifest))
    }

    /// Run tuist command with given args
    async fn tuist(&mut self, args: &[&str]) -> Result<()> {
        let mut process = Process::new(which("tuist")?);

        process.args(args);
        process.current_dir(self.root());

        let (success, _) = consume_and_log(Box::pin(process.spawn_and_stream()?)).await;
        if !success {
            return Err(Error::Generate);
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Project for TuistProject {
    async fn new(root: &PathBuf, logger: &Arc<Broadcast>) -> Result<Self> {
        let mut watchignore = generate_watchignore(root).await;

        watchignore.extend([
            "**/*.xcodeproj/**".into(),
            "**/*.xcworkspace/**".into(),
            "**/Tuist/Dependencies/**".into(),
        ]);

        let mut project = Self {
            root: root.clone(),
            watchignore,
            num_clients: 1,
            ..Self::default()
        };

        let (xcodeproj_path, manifest_path) = match project.xcodeproj_paths()? {
            (Some(xcodeproj_path), Some(manifest_path)) => (xcodeproj_path, manifest_path),
            (Some(_), None) => {
                project.tuist(&["edit", "--permanent"]).await?;

                let (a, b) = project.xcodeproj_paths()?;
                (a.unwrap(), b.unwrap())
            }
            (None, Some(_)) => {
                project.tuist(&["generate", "--no-open"]).await?;

                let (a, b) = project.xcodeproj_paths()?;
                (a.unwrap(), b.unwrap())
            }
            _ => {
                log::info!("no xcodeproj found at {root:?}");

                project.generate(logger).await?;

                project.targets = project.xcodeproj.targets_platform();
                project.manifest_files = project.manifest.build_file_names();

                log::info!("[{}] targets: {:?}", project.name(), project.targets());

                return Ok(project);
            }
        };

        project.manifest = XCodeProject::new(&manifest_path)?;
        project.manifest_path = manifest_path;
        project.manifest_files = project.manifest.build_file_names();

        project.xcodeproj = XCodeProject::new(&xcodeproj_path)?;
        project.xcodeproj_path = xcodeproj_path;
        project.targets = project.xcodeproj.targets_platform();

        log::info!("[{}] targets: {:?}", project.name(), project.targets());

        Ok(project)
    }
}
