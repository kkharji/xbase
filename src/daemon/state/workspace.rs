#[cfg(feature = "daemon")]
use crate::daemon::nvim::Nvim;

#[cfg(feature = "proc")]
use crate::util::proc;

#[cfg(feature = "daemon")]
use anyhow::{Context, Result};

#[cfg(feature = "xcodegen")]
use crate::xcodegen;

#[cfg(feature = "xcodegen")]
use std::collections::HashMap;

#[cfg(feature = "async")]
use tokio::task::JoinHandle;

use std::path::PathBuf;

use super::Project;

/// Managed Workspace
#[derive(Debug)]
pub struct Workspace {
    /// Project root path
    pub root: PathBuf,
    /// Project.yml base content
    pub project: Project,
    /// Active clients pids connect for the xcodebase workspace.
    #[cfg(feature = "daemon")]
    pub clients: HashMap<i32, Nvim>,
    #[cfg(feature = "daemon")]
    pub watch: Option<(crate::daemon::WatchStart, JoinHandle<Result<()>>)>,
}

#[cfg(feature = "daemon")]
impl Workspace {
    /// Create new workspace from a path representing project root.
    /// TODO: Support projects with .xproj as well as xcworkspace
    /// TODO: Ensure .compile and BuildServer.json exists
    pub async fn new(root: &str) -> Result<Self> {
        let root = PathBuf::from(root);
        let project = Project::new(&root)
            .await
            .context("Fail to create xcodegen project.")?;

        let workspace = Self {
            root,
            project,
            watch: None,
            clients: Default::default(),
        };

        if !workspace.root.join(".compile").is_file() {
            tracing::info!(".compile doesn't exist, regenerating ...");
            workspace.generate_compiliation_db().await?
        }
        Ok(workspace)
    }

    /// Regenerate compiled commands and xcodeGen if project.yml exists
    #[cfg(feature = "watcher")]
    pub async fn on_directory_change(
        &mut self,
        path: PathBuf,
        _event: &notify::EventKind,
    ) -> Result<()> {
        self.update_xcodeproj(path).await?;
        self.ensure_server_config().await?;
        self.generate_compiliation_db().await?;
        Ok(())
    }

    async fn generate_compiliation_db(&self) -> Result<()> {
        #[cfg(feature = "compilation")]
        {
            use crate::compile::CompilationDatabase;
            use tap::Pipe;
            use tokio_stream::StreamExt;
            use xcodebuild::parser::Step;

            let steps = self
                .project
                .fresh_build()
                .await?
                .collect::<Vec<Step>>()
                .await;

            CompilationDatabase::generate_from_steps(&steps)
                .await?
                .pipe(|cmd| serde_json::to_vec_pretty(&cmd.0))?
                .pipe(|json| tokio::fs::write(self.root.join(".compile"), json))
                .await
                .context("Write CompileCommands")?;
        }
        Ok(())
    }

    /// Ensure that buildServer.json exists in root directory.
    pub async fn ensure_server_config(&self) -> Result<()> {
        use tokio::fs::File;
        use tokio::io::AsyncWriteExt;

        let path = self.root.join("buildServer.json");
        if tokio::fs::File::open(&path).await.is_ok() {
            return Ok(());
        }

        tracing::info!("Creating {:?}", path);

        let mut file = tokio::fs::File::create(path).await?;
        let config = serde_json::json! ({
            "name": "XcodeBase Server",
            // FIXME: Point to user xcode-build-server
            "argv": ["/Users/tami5/repos/neovim/XcodeBase.nvim/target/debug/xcodebase-server"],
            "version": "0.1",
            "bspVersion": "0.2",
            "languages": [
                "swift",
                "objective-c",
                "objective-cpp",
                "c",
                "cpp"
            ]
        });

        AsyncWriteExt::write_all(&mut file, config.to_string().as_ref()).await?;
        File::sync_all(&file).await?;
        AsyncWriteExt::shutdown(&mut file).await?;

        Ok(())
    }

    /// Update .compile commands
    #[cfg(feature = "xcodegen")]
    pub async fn update_xcodeproj(&mut self, path: PathBuf) -> Result<()> {
        if !crate::xcodegen::is_workspace(self) {
            return Ok(());
        }

        tracing::info!("Updating {}.xcodeproj", self.name());

        let mut retry_count = 0;
        while retry_count < 3 {
            if let Ok(code) = xcodegen::generate(&self.root).await {
                if code.success() {
                    if path
                        .file_name()
                        .ok_or_else(|| anyhow::anyhow!("Fail to get filename from {:?}", path))?
                        .eq("project.yml")
                    {
                        tracing::info!("Updating State.{}.Project", self.name());
                        self.project = Project::new(&self.root).await?;
                        self.update_lua_state().await?;
                    }
                    return Ok(());
                }
            }
            retry_count += 1
        }

        anyhow::bail!("Fail to update_xcodeproj")
    }

    async fn update_lua_state(&mut self) -> Result<()> {
        tracing::info!("Updating nvim state");
        let update_project_state = self.project.nvim_update_state_script()?;

        let update_watch_state = format!(
            "require'xcodebase.watch'.is_watching = {}",
            self.is_watch_service_running()
        );

        for (pid, nvim) in self.clients.iter() {
            tracing::info!("Updating nvim for {pid}");
            nvim.exec_lua(&update_project_state, vec![]).await?;
            nvim.exec_lua(&update_watch_state, vec![]).await?;
        }

        Ok(())
    }

    pub fn update_clients(&mut self) {
        let name = self.project.name();
        self.clients.retain(|pid, _| {
            proc::exists(pid, || tracing::info!("[{}]: Remove Client: {pid}", name))
        })
    }

    /// Add new client to workspace (implicitly check if all other clients are stil valid).
    pub async fn add_client(&mut self, pid: i32, address: &str) -> Result<()> {
        // Remove no longer active clients
        self.update_clients();
        // NOTE: Implicitly assuming that pid is indeed a valid pid
        tracing::info!("[{}] Add Client: {pid}", self.name());
        self.clients.insert(pid, Nvim::new(address).await?);
        // Make sure that new client inherit other client state.
        self.update_lua_state().await?;
        Ok(())
    }

    /// Get nvim client
    pub fn get_client(&self, pid: &i32) -> Result<&Nvim> {
        match self.clients.get(pid) {
            Some(o) => Ok(o),
            None => anyhow::bail!("No nvim instance for {pid}"),
        }
    }

    /// Remove client from workspace
    pub fn remove_client(&mut self, pid: i32) -> usize {
        tracing::info!("[{}] Remove Client: {pid}", self.name());
        self.clients.retain(|&p, _| p != pid);
        self.clients.len()
    }

    /// Wrapper around Project.name:
    /// Returns project name
    pub fn name(&self) -> &str {
        self.project.name()
    }

    /// Wrapper around project.targets
    /// Returns all avaliable targets
    pub fn targets(&self) -> &crate::daemon::state::TargetMap {
        self.project.targets()
    }

    /// Get project target from project.targets using target_name
    pub fn get_target(&self, target_name: &str) -> Option<&crate::daemon::state::Target> {
        self.project.targets().get(target_name)
    }

    pub fn get_ignore_patterns(&self) -> Option<Vec<String>> {
        if crate::xcodegen::is_workspace(self) {
            return Some(self.project.config().ignore.clone());
        }
        return None;
    }
}

/// Watch Services
#[cfg(feature = "daemon")]
impl Workspace {
    /// Check if a watch service is running
    pub fn is_watch_service_running(&self) -> bool {
        self.watch.is_some()
    }

    /// Stop a watch service
    pub async fn stop_watch_service(&mut self) -> Result<()> {
        if let Some((_, ref mut handle)) = self.watch {
            handle.abort();
            handle.await.unwrap_err().is_cancelled();
            tracing::debug!("Watch service stopeed",);
        }
        Ok(())
    }

    /// Stop a watch service
    pub async fn start_watch_service(
        &mut self,
        watch_req: crate::daemon::WatchStart,
        handle: JoinHandle<Result<()>>,
    ) -> Result<()> {
        self.stop_watch_service().await?;
        self.watch = Some((watch_req, handle));
        Ok(())
    }
}
