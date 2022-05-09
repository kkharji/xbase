mod project;
mod workspace;

pub use project::*;
pub use workspace::*;

#[cfg(feature = "daemon")]
use anyhow::Result;

use std::collections::HashMap;

/// Main state
#[derive(Default, Debug)]
pub struct DaemonStateData {
    /// Manged workspaces
    pub workspaces: HashMap<String, Workspace>,
    /// Connected clients
    pub clients: Vec<i32>,
    // Current System. This is required mainly to check for
    #[cfg(feature = "async")]
    pub watchers: HashMap<String, tokio::task::JoinHandle<Result<()>>>,
}

#[cfg(feature = "async")]
pub type DaemonState = std::sync::Arc<tokio::sync::Mutex<DaemonStateData>>;

#[cfg(feature = "daemon")]
impl DaemonStateData {
    pub fn get_workspace(&self, root: &str) -> Result<&Workspace> {
        match self.workspaces.get(root) {
            Some(o) => Ok(o),
            None => anyhow::bail!("No workspace for {}", root),
        }
    }

    pub fn get_mut_workspace(&mut self, root: &str) -> Result<&mut Workspace> {
        match self.workspaces.get_mut(root) {
            Some(o) => Ok(o),
            None => anyhow::bail!("No workspace for {}", root),
        }
    }

    pub async fn add_workspace(&mut self, root: &str, pid: i32, address: &str) -> Result<()> {
        // TODO: Support projects with .xproj as well as xcworkspace

        if self.workspaces.contains_key(root) {
            let ws = self.get_mut_workspace(root).unwrap();
            return ws.add_client(pid, address).await;
        }

        let workspace = Workspace::new(root, pid, address).await?;
        let root = root.to_string();

        self.workspaces.insert(root, workspace);

        tracing::trace!("{:#?}", self);

        Ok(())
    }

    // Remove remove client from workspace and the workspace if it's this client was the last one.
    pub async fn remove_workspace(&mut self, root: &str, pid: i32) -> Result<()> {
        let mut name = None;

        if let Some(workspace) = self.workspaces.get_mut(root) {
            let clients_len = workspace.remove_client(pid);
            clients_len
                .eq(&0)
                .then(|| name = workspace.project.name().to_string().into());
        } else {
            tracing::error!("'{root}' is not a registered workspace!");
        }

        if let Some(name) = name {
            tracing::info!("Dropping [{}] {:?}", name, root);
            self.workspaces.remove(root);
        }

        Ok(())
    }

    pub fn validate(&mut self) {
        use crate::util::proc::exists as pid_exists;
        self.clients
            .retain(|pid| pid_exists(pid, || tracing::info!("Removing {pid}")));
        self.workspaces
            .iter_mut()
            .for_each(|(_, ws)| ws.update_clients())
    }
}
