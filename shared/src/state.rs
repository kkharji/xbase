use crate::tracing::info;
use crate::workspace::Workspace;
use anyhow::{Ok, Result};
use libproc::libproc::proc_pid;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::trace;

/// Main state
#[derive(Debug)]
pub struct State {
    /// Manged workspaces
    pub workspaces: HashMap<String, Workspace>,
    /// Connect clients
    pub clients: Vec<i32>,
    // Current System. This is required mainly to check for
}

pub type SharedState = Arc<Mutex<State>>;

impl State {
    pub fn new() -> Result<SharedState> {
        let state = State {
            workspaces: HashMap::new(),
            clients: vec![],
        };
        Ok(Arc::new(Mutex::new(state)))
    }

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
            Some(workspace) => {
                info!("Updated '{root}' Workspace ..");
                workspace.add_client(pid)
            }
            None => {
                info!("Added '{root}' Workspace ..");
                self.workspaces.insert(
                    root.to_string(),
                    Workspace::new_with_client(&root, pid).await?,
                );
            }
        };

        trace!("{:#?}", self);
        Ok(())
    }
}
