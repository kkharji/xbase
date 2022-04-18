mod project;
mod workspace;

pub use project::*;
pub use workspace::*;

#[cfg(feature = "daemon")]
use anyhow::Result;

use std::collections::HashMap;

/// Main state
#[derive(Default, Debug)]
pub struct State {
    /// Manged workspaces
    pub workspaces: HashMap<String, Workspace>,
    /// Connected clients
    pub clients: Vec<i32>,
    // Current System. This is required mainly to check for
    #[cfg(feature = "async")]
    pub watchers: HashMap<String, tokio::task::JoinHandle<Result<()>>>,
}

#[cfg(feature = "async")]
pub type SharedState = std::sync::Arc<tokio::sync::Mutex<State>>;

#[cfg(feature = "daemon")]
impl State {
    pub fn update_clients(&mut self) {
        self.clients
            .retain(|pid| crate::util::proc::exists(pid, || tracing::info!("Removeing {pid}")));

        self.workspaces
            .iter_mut()
            .for_each(|(_, ws)| ws.update_clients())
    }

    pub async fn add_workspace(&mut self, root: &str, pid: i32) -> Result<()> {
        match self.workspaces.get_mut(root) {
            Some(workspace) => workspace.add_client(pid),
            None => {
                let workspace = {
                    let root: &str = &root;
                    let mut ws = Workspace::new(&root).await?;
                    ws.add_client(pid);
                    ws
                };

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
            anyhow::bail!("workspace with '{root}' with given pid {pid} doesn't exist")
        }
        if let Some(name) = name {
            tracing::info!("Dropping [{}] {:?}", name, root);
            self.workspaces.remove(root);
        }
        Ok(())
    }
}
