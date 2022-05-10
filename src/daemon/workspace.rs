use super::WatchStart;
use crate::compile;
use crate::nvim::{Nvim, NvimClients};
use crate::types::{Project, Target};
use crate::util::proc_exists;
use anyhow::Result;
use std::collections::HashSet;
use std::path::PathBuf;
use tokio::task::JoinHandle;

/// Managed Workspace
#[derive(Debug)]
pub struct Workspace {
    /// Project root path
    pub root: PathBuf,
    /// Project.yml base content
    pub project: Project,
    /// Active clients pids connect for the xcodebase workspace.
    pub clients: NvimClients,
    pub watch: Option<(crate::daemon::WatchStart, JoinHandle<Result<()>>)>,
    pub watchers: HashSet<(WatchStart, JoinHandle<Result<()>>)>,
}

impl Workspace {
    pub async fn new(root: &str, pid: i32, address: &str) -> Result<Self> {
        let root_path = std::path::PathBuf::from(root);
        let project = anyhow::Context::context(
            Project::new(&root_path).await,
            "Fail to create xcodegen project.",
        )?;
        let name = project.name().to_string();

        let mut ws = Workspace {
            root: root_path,
            project,
            watch: None,
            clients: NvimClients::new(name),
            watchers: Default::default(),
        };

        if !ws.root.join(".compile").is_file() {
            tracing::info!(".compile doesn't exist, regenerating ...");
            compile::update_compilation_file(&ws.root).await?;
        }

        tracing::info!("New Workspace: {:?}", ws.project.name());
        tracing::trace!("{:?}", ws);

        ws.add_nvim_client(pid, address).await?;

        tracing::info!("Managing [{}] {:?}", ws.project.name(), root);
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
        self.clients.retain(|pid, _| {
            proc_exists(pid, || tracing::info!("[{}]: Remove Client: {pid}", name))
        })
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
        tracing::info!("[{}] Remove Client: {pid}", self.name());
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

/// Watch Services
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
            self.sync_state().await?;
        }
        self.watch = None;
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
