#[cfg(feature = "proc")]
use crate::util::proc;

#[cfg(feature = "xcode")]
use crate::xcode;

#[cfg(feature = "daemon")]
use anyhow::Result;

use std::path::PathBuf;

use crate::Project;

#[cfg(feature = "xcodegen")]
use crate::xcodegen;

/// Managed Workspace
#[derive(Debug)]
pub struct Workspace {
    /// Project root path
    pub root: PathBuf,
    /// Project.yml base content
    pub project: Project,
    /// Active clients pids connect for the xcodebase workspace.
    pub clients: Vec<i32>,
}

#[cfg(feature = "daemon")]
impl Workspace {
    /// Create new workspace from a path representing project root.
    /// TODO: Support projects with .xproj as well as xcworkspace
    pub async fn new(root: &str) -> Result<Self> {
        use anyhow::Context;
        let root = PathBuf::from(root);

        let project = {
            let path = root.join("project.yml");
            if !path.exists() {
                anyhow::bail!("project.yaml doesn't exist in '{:?}'", root)
            }

            Project::new_from_project_yml(root.clone(), path)
                .await
                .context("Fail to create xcodegen project.")?
        };

        Ok(Self {
            root,
            project,
            clients: vec![],
        })
    }

    pub async fn new_with_client(root: &str, pid: i32) -> Result<Self> {
        let mut ws = Workspace::new(&root).await?;
        ws.add_client(pid);
        Ok(ws)
    }

    pub fn update_clients(&mut self) {
        let name = self.project.name();
        self.clients
            .retain(|pid| proc::exists(pid, || tracing::info!("[{}]: Remove Client: {pid}", name)))
    }

    /// Add new client to workspace (implicitly check if all other clients are stil valid).
    pub fn add_client(&mut self, pid: i32) {
        // Remove no longer active clients
        self.update_clients();
        // NOTE: Implicitly assuming that pid is indeed a valid pid
        tracing::info!("[{}] Add Client: {pid}", self.name());
        self.clients.push(pid)
    }

    /// Remove client from workspace
    pub fn remove_client(&mut self, pid: i32) -> usize {
        tracing::info!("[{}] Remove Client: {pid}", self.name());
        self.clients.retain(|&p| p != pid);
        self.clients.iter().count()
    }

    /// Wrapper around Project.name:
    /// Returns project name
    pub fn name(&self) -> &str {
        self.project.name()
    }

    /// Wrapper around project.targets
    /// Returns all avaliable targets
    pub fn targets(&self) -> &crate::TargetMap {
        self.project.targets()
    }

    /// Get project target from project.targets using target_name
    pub fn get_target(&self, target_name: &str) -> Option<&crate::Target> {
        self.project.targets().get(target_name)
    }

    /// Regenerate compiled commands and xcodeGen if project.yml exists
    #[cfg(all(feature = "xcode", feature = "watcher"))]
    pub async fn on_directory_change(
        &mut self,
        path: PathBuf,
        _event: notify::EventKind,
    ) -> Result<()> {
        if crate::xcodegen::is_workspace(self) {
            let is_config_file = path.file_name().unwrap().eq("project");
            // FIXME: should've been true
            tracing::debug!("is_config_file: {is_config_file}");

            self.update_xcodeproj(is_config_file).await?;
        }

        xcode::ensure_server_config_file(&self.root).await?;
        xcode::update_compiled_commands(&self.root, self.project.fresh_build().await?).await?;

        Ok(())
    }

    /// Update .compile commands
    #[cfg(feature = "xcodegen")]
    pub async fn update_xcodeproj(&mut self, update_config: bool) -> Result<()> {
        match xcodegen::generate(&self.root).await {
            Ok(msg) => {
                tracing::info!("Updated {}.xcodeproj", self.name());
                tracing::trace!("{:?}", msg);
                if update_config {
                    tracing::info!("Updated internal state.{}.project", self.name());
                    self.project = Project::new_from_project_yml(
                        self.root.clone(),
                        xcodegen::config_path(self),
                    )
                    .await?;
                }
                Ok(())
            }
            Err(e) => {
                tracing::error!("{:?}", e);
                Err(e)
            }
        }
    }

    pub fn get_ignore_patterns(&self) -> Option<Vec<String>> {
        if crate::xcodegen::is_workspace(self) {
            return Some(self.project.config().ignore.clone());
        }
        return None;
    }
}
