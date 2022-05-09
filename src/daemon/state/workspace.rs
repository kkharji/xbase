#[cfg(feature = "daemon")]
use crate::compile;

#[cfg(feature = "daemon")]
use crate::daemon::nvim::Nvim;

#[cfg(feature = "proc")]
use crate::util::proc;

#[cfg(feature = "daemon")]
use anyhow::Result;

#[cfg(feature = "daemon")]
use std::collections::HashMap;

#[cfg(feature = "async")]
use tokio::task::JoinHandle;

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
    #[cfg(feature = "daemon")]
    pub clients: HashMap<i32, Nvim>,
    #[cfg(feature = "daemon")]
    // TODO: support watching multiple targets at the same time
    //
    // One way is to change watch to watchers of (maybe hash set of WatchStart)
    //
    // Also to make it convinent, the hash set should contain pid as first item in union.
    pub watch: Option<(crate::daemon::WatchStart, JoinHandle<Result<()>>)>,
    // pub watchers:
}

#[cfg(feature = "daemon")]
impl Workspace {
    pub async fn new(root: &str, pid: i32, address: &str) -> Result<Self> {
        let root_path = std::path::PathBuf::from(root);
        let project = anyhow::Context::context(
            Project::new(&root_path).await,
            "Fail to create xcodegen project.",
        )?;

        let mut ws = Workspace {
            root: root_path,
            project,
            watch: None,
            clients: Default::default(),
        };

        if !ws.root.join(".compile").is_file() {
            tracing::info!(".compile doesn't exist, regenerating ...");
            compile::update_compilation_file(&ws.root).await?;
        }

        tracing::info!("New Workspace: {:?}", ws.project.name());
        tracing::trace!("{:?}", ws);

        ws.get_nvim_client(pid, address).await?;

        tracing::info!("Managing [{}] {:?}", ws.project.name(), root);
        Ok(ws)
    }

    /// Add new client to workspace (implicitly check if all other clients are stil valid).
    pub async fn get_nvim_client(&mut self, pid: i32, address: &str) -> Result<()> {
        // Remove no longer active clients
        self.update_nvim_clients();
        // NOTE: Implicitly assuming that pid is indeed a valid pid
        tracing::info!("[{}] Add Client: {pid}", self.name());
        let nvim = Nvim::new(address).await?;

        tracing::debug!("Update nvim state for project");
        let script = &self.project.nvim_update_state_script()?;
        nvim.exec_lua(script, vec![]).await?;

        self.clients.insert(pid, nvim);
        // Make sure that new client inherit other client state.
        self.update_lua_state().await?;
        Ok(())
    }

    pub fn update_nvim_clients(&mut self) {
        let name = self.project.name();
        self.clients.retain(|pid, _| {
            proc::exists(pid, || tracing::info!("[{}]: Remove Client: {pid}", name))
        })
    }

    pub async fn update_lua_state(&mut self) -> Result<()> {
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

    /// Get nvim client
    pub fn nvim(&self, pid: &i32) -> Result<&Nvim> {
        match self.clients.get(pid) {
            Some(o) => Ok(o),
            None => anyhow::bail!("No nvim instance for {pid}"),
        }
    }

    pub async fn message_all_nvim_instances(&self, msg: &str) {
        for (_, nvim) in self.clients.iter() {
            if let Err(e) = nvim.exec(msg, false).await {
                tracing::error!("Fail to echo message to nvim clients {e}")
            }
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

    /// Get project target from project.targets using target_name
    pub fn get_target(&self, target_name: &str) -> Option<&crate::daemon::state::Target> {
        self.project.targets().get(target_name)
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
            self.update_lua_state().await?;
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
