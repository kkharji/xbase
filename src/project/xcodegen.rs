use super::*;
use crate::util::fs::which;
use crate::watcher::Event;
use crate::Result;
use futures::StreamExt;
use process_stream::{Process, ProcessExt};
use serde::Serialize;
use std::{collections::HashMap, path::PathBuf};
use xcodeproj::XCodeProject;

#[derive(Debug, Serialize, Default)]
#[serde(default)]
pub struct XCodeGenProject {
    root: PathBuf,
    targets: HashMap<String, TargetInfo>,
    num_clients: i32,
    watchignore: Vec<String>,
    #[serde(skip)]
    xcodeproj: xcodeproj::XCodeProject,
}

impl ProjectData for XCodeGenProject {
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
impl ProjectCompile for XCodeGenProject {
    async fn update_compile_database(&self, broadcast: &Arc<Broadcast>) -> Result<()> {
        use xclog::XCCompilationDatabase as CC;

        let root = self.root();
        let name = self.root().name().unwrap();
        let cache_root = self.build_cache_root()?;
        let mut arguments = self.compile_arguments();
        let task = Task::new(TaskKind::Compile, &name, broadcast.clone());

        arguments.push(format!("SYMROOT={cache_root}"));
        task.debug(format!("xcodebuild {}", arguments.join(" ")));

        let xclogger = XCLogger::new(&root, &arguments)?;
        let compile_commands = xclogger.compile_commands.clone();

        let success = task
            .consume(Box::new(xclogger))?
            .recv()
            .await
            .unwrap_or_default();
        if success {
            let compile_db = CC::new(compile_commands.lock().await.to_vec());
            if compile_db.is_empty() {
                broadcast.warn("No compile command was generated!");
            }
            let json = serde_json::to_vec_pretty(&compile_db)?;
            tokio::fs::write(root.join(".compile"), &json).await?;
            broadcast.reload_lsp_server();
            Ok(())
        } else {
            Err(Error::Compile)
        }
    }
}

#[async_trait::async_trait]
impl ProjectGenerate for XCodeGenProject {
    fn should_generate(&self, event: &Event) -> bool {
        let is_config_file = event.file_name() == "project.yml";
        let is_content_update = event.is_content_update_event();
        let is_config_file_update = is_content_update && is_config_file;

        is_config_file_update
            || event.is_create_event()
            || event.is_remove_event()
            || event.is_rename_event()
    }

    /// Generate xcodeproj
    async fn generate(&mut self, broadcast: &Arc<Broadcast>) -> Result<()> {
        let mut process: Process = vec![which("xcodegen")?.as_str(), "generate", "-c"].into();
        let name = self.root().name().unwrap();
        let task = Task::new(TaskKind::Generate, &name, broadcast.clone());
        process.current_dir(self.root());

        let mut logs = process
            .spawn_and_stream()
            .context("Spawn xcodegen")?
            .collect::<Vec<_>>()
            .await;
        let success = logs.pop().unwrap().is_success().unwrap_or_default();

        if !success {
            let logs = logs.into_iter().map(|p| p.to_string()).collect::<Vec<_>>();

            for log in logs {
                tracing::error!("{log}");
                task.error(log)
            }
        }

        let xcodeproj_paths = self.get_xcodeproj_paths()?;

        if xcodeproj_paths.len() > 1 {
            let using = xcodeproj_paths[0].display();
            tracing::warn!("[{name}] Found more then on xcodeproj, using {using}",);
        }

        self.xcodeproj = XCodeProject::new(&xcodeproj_paths[0]).context("Reading Project")?;
        for (key, info) in self.xcodeproj.targets_info().into_iter() {
            if self.targets.contains_key(&key) {
                let existing_info = self.targets.get_mut(&key).unwrap();
                *existing_info = info.into();
            } else {
                self.targets.insert(key, info.into());
            }
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Project for XCodeGenProject {
    #[tracing::instrument(parent = None, name = "Project", skip_all, fields(name = root.name().unwrap(), kind = "xcodegen"))]
    async fn new(root: &PathBuf, broadcast: &Arc<Broadcast>) -> Result<Self> {
        tracing::info!("Processing");
        let mut watchignore = generate_watchignore(root).await;
        watchignore.extend(["**/*.xcodeproj/**".into(), "**/*.xcworkspace/**".into()]);

        let mut project = Self {
            root: root.clone(),
            watchignore,
            num_clients: 1,
            ..Self::default()
        };

        tracing::debug!("Searching for *.xcodeproj");
        let xcodeproj_paths = project.get_xcodeproj_paths()?;

        if xcodeproj_paths.len() > 1 {
            tracing::warn!(
                "Found more then one *.xcodeproj, using {:?}",
                xcodeproj_paths[0]
            );
        }

        if !xcodeproj_paths.is_empty() {
            let xcpath = &xcodeproj_paths[0];
            tracing::debug!("Using {}", xcpath.abbrv().unwrap().display());
            project.xcodeproj = XCodeProject::new(xcpath).context("Reading XCodeProject")?;
            tracing::debug!("Identifying targets");
            project.targets = project
                .xcodeproj
                .targets_info()
                .into_iter()
                .map(|(k, info)| (k, info.into()))
                .collect();
            tracing::debug!("Targets: {:?} ", project.targets);
        } else {
            tracing::info!("Generating xcodeproj ...");
            if let Err(err) = project.generate(broadcast).await {
                return Err(Error::Setup(
                    project.name().to_string(),
                    format!("Generation failure {err}"),
                ));
            };
        }

        tracing::info!("Created");
        Ok(project)
    }
}

#[async_trait::async_trait]
impl ProjectBuild for XCodeGenProject {}

#[async_trait::async_trait]
impl ProjectRun for XCodeGenProject {}
