use crate::compile;
use crate::nvim::{Nvim, NvimClients};
use crate::types::{Project, Target};
use crate::util::proc_exists;
use anyhow::Result;
use std::path::PathBuf;
use tracing::{info, trace};

/// Managed Workspace
#[derive(Debug)]
pub struct Workspace {
    /// Project root path
    pub root: PathBuf,
    /// Project.yml base content
    pub project: Project,
    /// Active clients pids connect for the xcodebase workspace.
    pub clients: NvimClients,
    /// Ignore Patterns
    pub ignore_patterns: Vec<String>, // pub watchers: TargetWatchers,
}

impl Workspace {
    pub async fn new(root: &str, pid: i32, address: &str) -> Result<Self> {
        let root_path = std::path::PathBuf::from(root);
        let project = anyhow::Context::context(
            Project::new(&root_path).await,
            "Fail to create xcodegen project.",
        )?;
        let name = project.name().to_string();
        let mut ignore_patterns: Vec<String> = vec![
            "**/.git/**".into(),
            "**/*.xcodeproj/**".into(),
            "**/.*".into(),
            "**/build/**".into(),
            "**/buildServer.json".into(),
        ];

        // Note: Add extra ignore patterns to `ignore` local config requires restarting daemon.
        ignore_patterns.extend(project.config().ignore.clone());

        let mut ws = Workspace {
            root: root_path,
            project,
            clients: NvimClients::new(name),
            ignore_patterns,
        };

        if !ws.root.join(".compile").is_file() {
            info!(".compile doesn't exist, regenerating ...");
            compile::update_compilation_file(&ws.root).await?;
        }

        info!("New Workspace: {:?}", ws.project.name());
        trace!("{:?}", ws);

        ws.add_nvim_client(pid, address).await?;

        info!("Managing [{}] {:?}", ws.project.name(), root);
        Ok(ws)
    }

    /// Add new client to workspace (implicitly check if all other clients are stil valid).
    pub async fn add_nvim_client(&mut self, pid: i32, address: &str) -> Result<()> {
        self.ensure_active_clients();
        self.clients.insert(pid, address).await?;
        self.sync_state().await?;
        Ok(())
    }

    /// Remove no longer active clients
    pub fn ensure_active_clients(&mut self) {
        let name = self.project.name();
        self.clients
            .retain(|pid, _| proc_exists(pid, || info!("[{}]: Remove Client: {pid}", name)))
    }

    /// Make sure that clients have identical state.
    pub async fn sync_state(&self) -> Result<()> {
        Nvim::sync_state_ws(&self).await?;
        Ok(())
    }

    /// Get nvim client
    pub fn nvim(&self, pid: &i32) -> Result<&Nvim> {
        self.clients.get(pid)
    }

    /// Remove client from workspace
    pub fn remove_client(&mut self, pid: i32) -> usize {
        info!("[{}] Remove Client: {pid}", self.name());
        self.clients.retain(|&p, _| p != pid);
        self.clients.len()
    }

    /// Wrapper around Project.name:
    /// Returns project name
    pub fn name(&self) -> &str {
        self.project.name()
    }

    /// Get project target from project.targets using target_name
    pub fn get_target(&self, target_name: &str) -> Option<&Target> {
        self.project.targets().get(target_name)
    }
}
