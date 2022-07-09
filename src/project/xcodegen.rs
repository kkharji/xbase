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

        self.on_compile_start(broadcast)?;

        arguments.push(format!("SYMROOT={cache_root}"));
        broadcast.log_debug(format!("[{name}] xcodebuild {}", arguments.join(" ")));

        let xclogger = XCLogger::new(&root, &arguments)?;
        let compile_commands = xclogger.compile_commands.clone();

        let success = broadcast
            .consume(Box::new(xclogger))?
            .recv()
            .await
            .unwrap_or_default();

        self.on_compile_finish(success, broadcast)?;

        let compile_db = CC::new(compile_commands.lock().await.to_vec());
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

        is_config_file_update
            || event.is_create_event()
            || event.is_remove_event()
            || event.is_rename_event()
    }

    /// Generate xcodeproj
    async fn generate(&mut self, broadcast: &Arc<Broadcast>) -> Result<()> {
        self.on_generate_start(broadcast)?;

        let mut process: Process = vec![which("xcodegen")?.as_str(), "generate", "-c"].into();
        process.current_dir(self.root());

        let mut logs = process.spawn_and_stream()?.collect::<Vec<_>>().await;
        let success = logs.pop().unwrap().is_success().unwrap_or_default();

        if !success {
            let logs = logs.into_iter().map(|p| p.to_string()).collect::<Vec<_>>();

            for log in logs {
                broadcast.log_error(log)
            }
        }
        self.on_generate_finish(success, broadcast)?;

        let xcodeproj_paths = self.get_xcodeproj_paths()?;
        let name = self.name();

        if xcodeproj_paths.len() > 1 {
            let using = xcodeproj_paths[0].display();
            tracing::warn!("[{name}] Found more then on xcodeproj, using {using}",);
        }

        self.xcodeproj = XCodeProject::new(&xcodeproj_paths[0])?;
        for (key, platform) in self.xcodeproj.targets_platform().into_iter() {
            if self.targets.contains_key(&key) {
                let info = self.targets.get_mut(&key).unwrap();
                info.platform = platform;
            } else {
                self.targets.insert(key, TargetInfo { platform });
            }
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Project for XCodeGenProject {
    async fn new(root: &PathBuf, broadcast: &Arc<Broadcast>) -> Result<Self> {
        let mut watchignore = generate_watchignore(root).await;
        watchignore.extend(["**/*.xcodeproj/**".into(), "**/*.xcworkspace/**".into()]);

        let mut project = Self {
            root: root.clone(),
            watchignore,
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

        if !xcodeproj_paths.is_empty() {
            project.xcodeproj = XCodeProject::new(&xcodeproj_paths[0])?;
            project.targets = project
                .xcodeproj
                .targets_platform()
                .into_iter()
                .map(|(k, platform)| (k, TargetInfo { platform }))
                .collect();
        } else {
            project.generate(broadcast).await?;
        }

        tracing::info!("[{}] targets: {:?}", project.name(), project.targets());
        Ok(project)
    }
}

#[async_trait::async_trait]
impl ProjectBuild for XCodeGenProject {}

#[async_trait::async_trait]
impl ProjectRun for XCodeGenProject {}
