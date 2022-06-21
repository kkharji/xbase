use super::*;
use crate::watch::Event;
use crate::{Error, Result};
use serde::Serialize;
use std::{collections::HashMap, path::PathBuf};
use xbase_proto::Client;
use xcodeproj::{pbxproj::PBXTargetPlatform, XCodeProject};

#[derive(Debug, Serialize, Default)]
#[serde(default)]
pub struct BareboneProject {
    root: PathBuf,
    targets: HashMap<String, PBXTargetPlatform>,
    clients: Vec<i32>,
    watchignore: Vec<String>,
    #[serde(skip)]
    xcodeproj: XCodeProject,
}

impl ProjectData for BareboneProject {
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
impl ProjectBuild for BareboneProject {}

#[async_trait::async_trait]
impl ProjectCompile for BareboneProject {
    async fn update_compile_database(&self) -> Result<()> {
        use xclog::XCCompilationDatabase as CC;

        let (name, root) = (self.name(), self.root());
        let cache_root = self.build_cache_root()?;
        let mut arguments = self.compile_arguments();

        arguments.extend([
            format!("SYMROOT={cache_root}"),
            "-project".into(),
            format!("{name}.xcodeproj"),
        ]);

        let compile_commands = CC::generate(&root, &arguments).await?.to_vec();
        let json = serde_json::to_vec_pretty(&compile_commands)?;

        tokio::fs::write(root.join(".compile"), &json).await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl ProjectGenerate for BareboneProject {
    fn should_generate(&self, event: &Event) -> bool {
        event.is_create_event() || event.is_remove_event()
    }

    async fn generate(&mut self) -> Result<()> {
        log::error!("New File created or removed but generate barebone project is not supported");

        Ok(())
    }
}

#[async_trait::async_trait]
impl Project for BareboneProject {
    async fn new(client: &Client) -> Result<Self> {
        let Client { root, pid, .. } = client;

        log::info!("Project Type: Barebone");

        let mut project = Self {
            root: root.clone(),
            watchignore: generate_watchignore(root).await,
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

        if xcodeproj_paths.is_empty() {
            return Err(Error::ProjectError("No XcodeProjectFound!".into()));
        };

        project.xcodeproj = XCodeProject::new(&xcodeproj_paths[0])?;
        project.targets = project.xcodeproj.targets_platform();

        log::debug!("Project Name: {}", project.name());
        log::debug!("Project Targets: {:?}", project.targets());

        Ok(project)
    }
}
