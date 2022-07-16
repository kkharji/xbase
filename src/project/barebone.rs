use super::*;
use crate::*;
use serde::Serialize;
use std::{collections::HashMap, path::PathBuf};
use xcodeproj::XCodeProject;

#[derive(Debug, Serialize, Default)]
#[serde(default)]
pub struct BareboneProject {
    root: PathBuf,
    targets: HashMap<String, TargetInfo>,
    num_clients: i32,
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

    fn targets(&self) -> &HashMap<String, TargetInfo> {
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
impl ProjectBuild for BareboneProject {}

#[async_trait::async_trait]
impl ProjectRun for BareboneProject {}

#[async_trait::async_trait]
impl ProjectCompile for BareboneProject {
    async fn update_compile_database(&self, broadcast: &Arc<Broadcast>) -> Result<()> {
        let (name, root) = (self.name(), self.root());
        let cache_root = self.build_cache_root()?;
        let mut args = self.compile_arguments();

        args.push(format!("SYMROOT={cache_root}"));

        let xcworkspace = format!("{name}.xcworkspace");

        if self.root().join(&xcworkspace).exists() {
            args.extend_from_slice(&[
                "-workspace".into(),
                xcworkspace,
                "-scheme".into(),
                name.into(),
            ]);
        } else {
            args.extend_from_slice(&["-project".into(), format!("{name}.xcodeproj")]);
        }
        let task = Task::new(TaskKind::Compile, name, broadcast.clone());

        // broadcast.log_debug(format!("[{name}] xcodebuild {}", args.join(" ")));
        let xclogger = XCLogger::new(&root, &args)?;
        let xccommands = xclogger.compile_commands.clone();
        let mut recv = task.consume(Box::new(xclogger))?;

        // self.on_compile_finish(recv.recv().await.unwrap_or_default(), broadcast)?;
        if recv.recv().await.unwrap_or_default() {
            let json = serde_json::to_vec_pretty(&xccommands.lock().await.to_vec())?;
            tokio::fs::write(root.join(".compile"), &json).await?;
            broadcast.reload_lsp_server();
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl ProjectGenerate for BareboneProject {
    fn should_generate(&self, _event: &Event) -> bool {
        false
    }

    async fn generate(&mut self, _logger: &Arc<Broadcast>) -> Result<()> {
        tracing::error!(
            "New File created or removed but generate barebone project is not supported"
        );

        Ok(())
    }
}

#[async_trait::async_trait]
impl Project for BareboneProject {
    async fn new(root: &PathBuf, _logger: &Arc<Broadcast>) -> Result<Self> {
        let mut project = Self {
            root: root.clone(),
            watchignore: generate_watchignore(root).await,
            num_clients: 1,
            ..Self::default()
        };

        let xcodeproj_paths = project.get_xcodeproj_paths()?;
        if xcodeproj_paths.len() > 1 {
            tracing::warn!(
                "Found more then on xcodeproj, using {:?}",
                xcodeproj_paths[0]
            );
        }

        if xcodeproj_paths.is_empty() {
            return Err(Error::DefinitionLocating);
        };

        project.xcodeproj = XCodeProject::new(&xcodeproj_paths[0])?;
        project.targets = project
            .xcodeproj
            .targets_platform()
            .into_iter()
            .map(|(k, platform)| {
                (
                    k,
                    TargetInfo {
                        platform: platform.to_string(),
                    },
                )
            })
            .collect();

        tracing::info!("targets: {:?}", project.targets());
        Ok(project)
    }
}
