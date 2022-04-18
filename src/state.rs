use crate::workspace::Workspace;
use anyhow::{bail, Ok, Result};
use libproc::libproc::proc_pid;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::trace;

/// Main state
#[derive(Debug, Default)]
pub struct State {
    /// Manged workspaces
    pub workspaces: HashMap<String, Workspace>,
    /// Connect clients
    pub clients: Vec<i32>,
    // Current System. This is required mainly to check for
    pub watchers: HashMap<String, JoinHandle<Result<()>>>,
}

pub type SharedState = Arc<Mutex<State>>;

impl State {
    pub fn update_clients(&mut self) {
        self.clients.retain(|&pid| {
            if proc_pid::name(pid).is_err() {
                tracing::trace!("Removeing {pid}");
                false
            } else {
                true
            }
        });

        self.workspaces
            .iter_mut()
            .for_each(|(_, ws)| ws.update_clients())
    }

    pub async fn add_workspace(&mut self, root: &str, pid: i32) -> Result<()> {
        match self.workspaces.get_mut(root) {
            Some(workspace) => workspace.add_client(pid),
            None => {
                let workspace = Workspace::new_with_client(&root, pid).await?;

                tracing::info!("Managing [{}] {:?}", workspace.project.name(), root);

                self.workspaces.insert(root.to_string(), workspace);
            }
        };

        // Print New state
        trace!("{:#?}", self);
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
            bail!("workspace with '{root}' with given pid {pid} doesn't exist")
        }
        if let Some(name) = name {
            tracing::info!("Dropping [{}] {:?}", name, root);
            self.workspaces.remove(root);
        }
        Ok(())
    }
}
