use super::*;
use crate::util::fs::which;
use crate::watcher::Event;
use crate::{Error, Result};
use futures::future::try_join_all;
use futures::StreamExt;
use process_stream::{Process, ProcessExt};
use serde::Serialize;
use std::{collections::HashMap, path::PathBuf};
use tap::Pipe;
use xcodeproj::XCodeProject;

#[derive(Debug, Serialize, Default)]
#[serde(default)]
pub struct TuistProject {
    root: PathBuf,
    targets: HashMap<String, TargetInfo>,
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
impl ProjectCompile for TuistProject {
    async fn update_compile_database(&self, broadcast: &Arc<Broadcast>) -> Result<()> {
        use xclog::XCCompileCommand as C;

        let name = self.name();
        let root = self.root();
        let cache_root = self.build_cache_root()?;
        let args = self.compile_arguments();
        let mut tasks_recvs = vec![];
        let mut xccommands: Vec<Arc<Mutex<Vec<C>>>> = vec![];
        let task = Task::new(TaskKind::Compile, self.name(), broadcast.clone());

        {
            // Compile manifests
            let mut args = args.clone();

            args.extend_from_slice(&[
                format!("SYMROOT={cache_root}_tuist"),
                "-workspace".into(),
                "Manifests.xcworkspace".into(),
                "-scheme".into(),
                "Manifests".into(),
            ]);

            let xclogger = XCLogger::new(&root, &args)?;
            xccommands.push(xclogger.compile_commands.clone());
            tasks_recvs.push(task.consume(Box::new(xclogger))?);

            let argsstr = args.join(" ");
            tracing::info!("Building Manifest ...");
            tracing::trace!("\n\n xcodebuild {argsstr}\n\n");
            task.debug(format!("[{name}] {argsstr}"));
        }

        for scheme in self.xcodeproj.schemes().into_iter() {
            let mut args = args.clone();

            args.extend_from_slice(&[
                format!("SYMROOT={cache_root}"),
                "-workspace".into(),
                format!("{name}.xcworkspace"),
                "-scheme".into(),
                scheme.name.clone(),
            ]);

            let xclogger = XCLogger::new(&root, &args)?;
            xccommands.push(xclogger.compile_commands.clone());
            tasks_recvs.push(task.consume(Box::new(xclogger))?);
            let argsstr = args.join(" ");
            tracing::info!("Building {} ...", scheme.name);
            tracing::trace!("\n\n xcodebuild {argsstr}\n\n");
            task.debug(format!("[{name}] {argsstr}"));
        }

        let _all_pass = tasks_recvs
            .into_iter()
            .map(|mut t| tokio::spawn(async move { t.recv().await.unwrap_or_default() }))
            .pipe(try_join_all)
            .await
            .unwrap_or_default()
            .into_iter()
            .all(|f| f);

        let mut xccommands = xccommands
            .into_iter()
            .map(|l| tokio::spawn(async move { l.lock().await.to_vec() }))
            .pipe(try_join_all)
            .await
            .unwrap_or_default()
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        xccommands.dedup();
        if xccommands.is_empty() {
            broadcast.warn("No compile command was generated!");
        }

        let json = serde_json::to_vec_pretty(&xccommands)?;
        tokio::fs::write(root.join(".compile"), &json).await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl ProjectGenerate for TuistProject {
    fn should_generate(&self, event: &Event) -> bool {
        tracing::trace!("manifest files {:?}", self.manifest_files);
        let is_config_file = self.manifest_files.contains(event.file_name());
        let is_content_update = event.is_content_update_event();
        let is_config_file_update = is_content_update && is_config_file;

        is_config_file_update
            || event.is_create_event()
            || event.is_remove_event()
            || event.is_rename_event()
    }

    /// Generate xcodeproj
    async fn generate(&mut self, broadcast: &Arc<Broadcast>) -> Result<()> {
        let task = Task::new(TaskKind::Generate, self.name(), broadcast.clone());
        self.tuist(&task, &["edit", "--permanent"]).await?;
        self.tuist(&task, &["generate", "--no-open"]).await?;

        let (xcodeproj_path, manifest_path) = self.xcodeproj_paths()?;
        let (xcodeproj_path, manifest_path) = (xcodeproj_path.unwrap(), manifest_path.unwrap());

        self.manifest = XCodeProject::new(&manifest_path)?;
        self.manifest_path = manifest_path;
        self.xcodeproj = XCodeProject::new(&xcodeproj_path)?;
        self.xcodeproj_path = xcodeproj_path;

        for (key, info) in self.xcodeproj.targets_info().into_iter() {
            if self.targets.contains_key(&key) {
                let existing_info = self.targets.get_mut(&key).unwrap();
                *existing_info = info.into()
            } else {
                self.targets.insert(key, info.into());
            }
        }

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
            tracing::warn!(
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
    async fn tuist(&mut self, task: &Task, args: &[&str]) -> Result<()> {
        let mut process = Process::new(which("tuist")?);

        process.args(args);
        process.current_dir(self.root());
        let mut logs = process.spawn_and_stream()?.collect::<Vec<_>>().await;
        let success = logs.pop().unwrap().is_success().unwrap_or_default();

        if !success {
            task.error("Tuist Project Generation failed ");
            let logs = logs.into_iter().map(|p| p.to_string()).collect::<Vec<_>>();

            for log in logs {
                task.error(log)
            }

            return Err(Error::Generate);
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Project for TuistProject {
    async fn new(root: &PathBuf, broadcast: &Arc<Broadcast>) -> Result<Self> {
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
                let task = Task::new(TaskKind::Generate, "Manifest", broadcast.clone());
                project.tuist(&task, &["edit", "--permanent"]).await?;

                let (a, b) = project.xcodeproj_paths()?;
                (a.unwrap(), b.unwrap())
            }
            (None, Some(_)) => {
                let task = Task::new(TaskKind::Generate, project.name(), broadcast.clone());
                project.tuist(&task, &["generate", "--no-open"]).await?;

                let (a, b) = project.xcodeproj_paths()?;
                (a.unwrap(), b.unwrap())
            }
            _ => {
                tracing::info!("no xcodeproj found at {root:?}");

                project.generate(broadcast).await?;

                tracing::info!("[{}] targets: {:?}", project.name(), project.targets());

                return Ok(project);
            }
        };

        project.manifest = XCodeProject::new(&manifest_path)?;
        project.manifest_path = manifest_path;
        project.manifest_files = project.manifest.build_file_names();

        project.xcodeproj = XCodeProject::new(&xcodeproj_path)?;
        project.xcodeproj_path = xcodeproj_path;
        project.targets = project
            .xcodeproj
            .targets_info()
            .into_iter()
            .map(|(k, info)| (k, info.into()))
            .collect();

        tracing::info!("[{}] targets: {:?}", project.name(), project.targets());

        Ok(project)
    }
}

#[async_trait::async_trait]
impl ProjectBuild for TuistProject {}

#[async_trait::async_trait]
impl ProjectRun for TuistProject {}
