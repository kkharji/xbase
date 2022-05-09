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
    pub fn update_clients(&mut self) {
        self.clients
            .retain(|pid| crate::util::proc::exists(pid, || tracing::info!("Removing {pid}")));

        self.workspaces
            .iter_mut()
            .for_each(|(_, ws)| ws.update_clients())
    }

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
        match self.workspaces.get_mut(root) {
            Some(workspace) => workspace.add_client(pid, address).await?,
            None => {
                let workspace = {
                    let root: &str = &root;
                    let mut ws = Workspace::new(&root).await?;
                    tracing::info!("New Workspace: {:?}", ws.project.name());
                    tracing::trace!("{:?}", ws);
                    ws.add_client(pid, address).await?;
                    ws
                };

                if let Some(nvim) = workspace.clients.get(&pid) {
                    tracing::debug!("Update nvim state for project");
                    nvim.exec_lua(&workspace.project.nvim_update_state_script()?, vec![])
                        .await?;
                }

                tracing::info!("Managing [{}] {:?}", workspace.project.name(), root);

                self.workspaces.insert(root.to_string(), workspace);
            }
        };

        // Print New state
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
}
