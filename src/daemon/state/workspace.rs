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
        Ok(Self {
            root,
            project,
            clients: Default::default(),
        })
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
        Ok(())
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

    /// Regenerate compiled commands and xcodeGen if project.yml exists
    #[cfg(feature = "watcher")]
    pub async fn on_directory_change(
        &mut self,
        path: PathBuf,
        _event: &notify::EventKind,
    ) -> Result<()> {
        use tap::Pipe;

        if crate::xcodegen::is_workspace(self) {
            self.update_xcodeproj(
                path.file_name()
                    .ok_or_else(|| anyhow::anyhow!("Fail to get filename from {:?}", path))?
                    .eq("project.yml"),
            )
            .await?;
        }

        #[cfg(feature = "xcode")]
        crate::xcode::ensure_server_config_file(&self.root).await?;

        // TODO: ensure .compile file on on_directory_change and in workspace initialization

        #[cfg(feature = "compilation")]
        self.project
            .fresh_build()
            .await?
            .pipe(crate::compile::CompilationDatabase::from_logs)
            .pipe(|cmd| serde_json::to_vec_pretty(&cmd.0))?
            .pipe(|json| tokio::fs::write(self.root.join(".compile"), json))
            .await
            .context("Write CompileCommands")?;

        Ok(())
    }

    /// Update .compile commands
    #[cfg(feature = "xcodegen")]
    pub async fn update_xcodeproj(&mut self, update_config: bool) -> Result<()> {
        tracing::info!("Updating {}.xcodeproj", self.name());

        let mut retry_count = 0;
        while retry_count < 3 {
            if let Ok(code) = xcodegen::generate(&self.root).await {
                if code.success() {
                    if update_config {
                        tracing::info!("Updating State.{}.Project", self.name());
                        self.project = Project::new(&self.root).await?;
                    }
                    return Ok(());
                }
            }
            retry_count += 1
        }

        anyhow::bail!("Fail to update_xcodeproj")
    }

    pub fn get_ignore_patterns(&self) -> Option<Vec<String>> {
        if crate::xcodegen::is_workspace(self) {
            return Some(self.project.config().ignore.clone());
        }
        return None;
    }
}
