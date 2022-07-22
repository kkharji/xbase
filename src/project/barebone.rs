use super::*;
use crate::*;
use futures::future::try_join_all;
use serde::Serialize;
use std::{collections::HashMap, path::PathBuf};
use tap::Pipe;
use xclog::XCCompileCommand;
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
        let mut tasks_recvs = vec![];
        let mut xccommands: Vec<Arc<Mutex<Vec<XCCompileCommand>>>> = vec![];

        let task = Task::new(TaskKind::Compile, name, broadcast.clone());

        args.push(format!("SYMROOT={cache_root}"));

        let xcworkspace = format!("{name}.xcworkspace");

        if self.root().join(&xcworkspace).exists() {
            for scheme in self.xcodeproj.schemes().iter() {
                let mut args = args.clone();
                args.extend_from_slice(&[
                    "-workspace".into(),
                    xcworkspace.clone(),
                    "-scheme".into(),
                    scheme.name.clone(),
                ]);
                let xclogger = XCLogger::new(&root, &args)?;
                xccommands.push(xclogger.compile_commands.clone());
                tasks_recvs.push(task.consume(Box::new(xclogger))?);
            }
        } else {
            args.extend_from_slice(&["-project".into(), format!("{name}.xcodeproj")]);
            let xclogger = XCLogger::new(&root, &args)?;
            xccommands.push(xclogger.compile_commands.clone());
            tasks_recvs.push(task.consume(Box::new(xclogger))?);
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

        let json = serde_json::to_vec_pretty(&xccommands)?;
        tokio::fs::write(root.join(".compile"), &json).await?;

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
            .targets_info()
            .into_iter()
            .map(|(k, info)| (k, info.into()))
            .collect();

        tracing::info!("targets: {:?}", project.targets());
        Ok(project)
    }
}
