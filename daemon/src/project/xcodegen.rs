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
pub struct XCodeGenProject {
    root: PathBuf,
    targets: HashMap<String, PBXTargetPlatform>,
    clients: Vec<i32>,
    watchignore: Vec<String>,
    #[serde(skip)]
    xcodeproj: xcodeproj::XCodeProject,
    xcodeproj_paths: Vec<PathBuf>,
}

impl ProjectData for XCodeGenProject {
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
impl ProjectBuild for XCodeGenProject {}

#[async_trait::async_trait]
impl ProjectCompile for XCodeGenProject {
    async fn update_compile_database(&self) -> Result<()> {
        use xclog::XCCompilationDatabase as CC;

        let root = self.root();
        let cache_root = self.build_cache_root()?;
        let mut arguments = self.compile_arguments();

        arguments.push(format!("SYMROOT={cache_root}"));

        log::debug!(
            "Updating compile database with: `xcodebuild {}`",
            arguments.join(" ")
        );

        let compile_db = CC::generate(&root, &arguments).await?;
        let json = serde_json::to_vec_pretty(&compile_db)?;
        tokio::fs::write(root.join(".compile"), &json).await?;

        Ok(())
    }
}
#[async_trait::async_trait]
impl ProjectGenerate for XCodeGenProject {
    fn should_generate(&self, event: &Event) -> bool {
        let is_config_file = event.file_name() == "project.yml";
        let is_content_update = event.is_content_update_event();
        let is_config_file_update = is_content_update && is_config_file;

        is_config_file_update || event.is_create_event() || event.is_remove_event()
    }

    /// Generate xcodeproj
    async fn generate(&mut self) -> Result<()> {
        log::info!("generating xcodeproj with xcodegen");

        let mut process: Process = vec![which("xcodegen")?.as_str(), "generate", "-c"].into();
        process.current_dir(self.root());

        let (success, logs) = consume_and_log(Box::pin(process.spawn_and_stream()?)).await;

        if success {
            let xcodeproj_paths = self.get_xcodeproj_paths()?;

            if xcodeproj_paths.len() > 1 {
                log::warn!(
                    "Found more then on xcodeproj, using {:?}",
                    xcodeproj_paths[0]
                );
            }
            self.xcodeproj = XCodeProject::new(&xcodeproj_paths[0])?;
            self.targets = self.xcodeproj.targets_platform();
        } else {
            return Err(Error::XCodeProjectGenerate(
                self.name().into(),
                logs.join("\n"),
            ));
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Project for XCodeGenProject {
    async fn new(client: &Client) -> Result<Self> {
        let Client { root, pid, .. } = client;

        log::info!("Project Type: XcodeGen");

        let mut watchignore = generate_watchignore(root).await;
        watchignore.extend(["**/*.xcodeproj/**".into(), "**/*.xcworkspace/**".into()]);

        let mut project = Self {
            root: root.clone(),
            watchignore,
            clients: vec![pid.clone()],
            ..Self::default()
        };

        let xcodeproj_paths = project.get_xcodeproj_paths()?;

        if xcodeproj_paths.len() > 1 {
            log::warn!(
                "Found more then on xcodeproj, using {:?}",
                xcodeproj_paths[0]
            );
        }

        if !project.xcodeproj_paths.is_empty() {
            project.xcodeproj = XCodeProject::new(&xcodeproj_paths[0])?;
            project.targets = project.xcodeproj.targets_platform();
        } else {
            log::info!("no xcodeproj found at {root:?}");
            project.generate().await?;
        }

        log::info!(
            "(name: {:?}, targets: {:?})",
            project.name(),
            project.targets()
        );
        Ok(project)
    }
}
