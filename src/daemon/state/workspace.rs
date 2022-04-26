#[cfg(feature = "proc")]
use crate::util::proc;

#[cfg(feature = "daemon")]
use anyhow::Result;

#[cfg(feature = "xcodegen")]
use crate::xcodegen;

use super::Project;
use std::path::PathBuf;

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

        // TODO: Ensure .compile and BuildServer.json exists

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
        use anyhow::Context;
        use tap::Pipe;

        if crate::xcodegen::is_workspace(self) {
            self.update_xcodeproj(path.file_name().unwrap().eq("project.yml"))
                .await?;
        }

        #[cfg(feature = "xcode")]
        crate::xcode::ensure_server_config_file(&self.root).await?;
        #[cfg(feature = "compilation")]
        self.project
            .fresh_build()
            .await?
            .pipe(|logs| crate::compile::CompilationDatabase::from_logs(logs))
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

        let retry_count = 0;
        while retry_count < 3 {
            if let Ok(code) = xcodegen::generate(&self.root).await {
                if code.success() {
                    if update_config {
                        tracing::info!("Updating State.{}.Project", self.name());
                        self.project = Project::new_from_project_yml(
                            self.root.clone(),
                            xcodegen::config_path(self),
                        )
                        .await?;
                    }
                    return Ok(());
                }
            }
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
