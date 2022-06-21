use super::*;
use crate::util::fs::which;
use crate::watch::Event;
use crate::{Error, Result};
use process_stream::Process;
use serde::Serialize;
use std::{collections::HashMap, path::PathBuf};
use xbase_proto::Client;
use xcodeproj::{pbxproj::PBXTargetPlatform, XCodeProject};

#[derive(Debug, Serialize, Default)]
#[serde(default)]
pub struct TuistProject {
    root: PathBuf,
    targets: HashMap<String, PBXTargetPlatform>,
    clients: Vec<i32>,
    watchignore: Vec<String>,
    #[serde(skip)]
    xcodeproj: XCodeProject,
    #[serde(skip)]
    xcodeproj_path: PathBuf,
    #[serde(skip)]
    manifest: XCodeProject,
    #[serde(skip)]
    manifest_path: PathBuf,
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

    fn clients(&self) -> &Vec<i32> {
        &self.clients
    }

    fn clients_mut(&mut self) -> &mut Vec<i32> {
        &mut self.clients
    }

    fn watchignore(&self) -> &Vec<String> {
        &self.watchignore
    }
}

#[async_trait::async_trait]
impl ProjectBuild for TuistProject {}

#[async_trait::async_trait]
impl ProjectCompile for TuistProject {
    async fn update_compile_database(&self) -> Result<()> {
        use xclog::XCCompilationDatabase as CC;
        use xclog::XCCompileCommand as C;

        let name = self.name();
        let root = self.root();
        let cache_root = self.build_cache_root()?;
        let arguments = self.compile_arguments();
        let mut compile_commands: Vec<C> = vec![];

        // Compile manifests
        {
            let mut arguments = arguments.clone();

            arguments.push(format!("SYMROOT={cache_root}_tuist"));
            arguments.push("-project".into());
            arguments.push("Manifests.xcodeproj".into());

            // TODO(tuist): generate build arguments for all manifest targets

            log::debug!(
                "Getting compile commands from : `xcodebuild {}`",
                arguments.join(" ")
            );

            compile_commands.extend(CC::generate(&root, &arguments).await?.to_vec());
        }

        // Compile Project
        {
            let mut arguments = arguments.clone();

            arguments.push(format!("SYMROOT={cache_root}"));
            arguments.push("-project".into());
            arguments.push(format!("{name}.xcodeproj"));

            compile_commands.extend(CC::generate(&root, &arguments).await?.to_vec());
        }

        let json = serde_json::to_vec_pretty(&compile_commands)?;
        tokio::fs::write(root.join(".compile"), &json).await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl ProjectGenerate for TuistProject {
    fn should_generate(&self, event: &Event) -> bool {
        // TODO(tuist): support regenrating on all manifest files
        let is_config_file = event.file_name() == "Project.swift";
        let is_content_update = event.is_content_update_event();
        let is_config_file_update = is_content_update && is_config_file;

        is_config_file_update || event.is_create_event() || event.is_remove_event()
    }

    /// Generate xcodeproj
    async fn generate(&mut self) -> Result<()> {
        log::info!("generating xcodeproj with tuist");

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

        let (success, logs) = consume_and_log(Box::pin(process.spawn_and_stream()?)).await;
        if !success {
            return Err(Error::XCodeProjectGenerate(
                "Project".into(),
                logs.join("\n"),
            ));
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Project for TuistProject {
    async fn new(client: &Client) -> Result<Self> {
        let Client { root, pid, .. } = client;

        log::debug!("Project Type: Tuist");

        let mut watchignore = generate_watchignore(root).await;

        watchignore.extend([
            "**/*.xcodeproj/**".into(),
            "**/*.xcworkspace/**".into(),
            "**/Tuist/Dependencies/**".into(),
        ]);

        let mut project = Self {
            root: root.clone(),
            watchignore,
            clients: vec![pid.clone()],
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

                project.generate().await?;

                project.targets = project.xcodeproj.targets_platform();

                log::debug!("Project Name: {}", project.name());
                log::debug!("Project Targets: {:?}", project.targets());

                return Ok(project);
            }
        };

        project.manifest = XCodeProject::new(&manifest_path)?;
        project.manifest_path = manifest_path;
        project.xcodeproj = XCodeProject::new(&xcodeproj_path)?;
        project.xcodeproj_path = xcodeproj_path;
        project.targets = project.xcodeproj.targets_platform();

        log::debug!("Project Name: {}", project.name());
        log::debug!("Project Targets: {:?}", project.targets());

        Ok(project)
    }
}
