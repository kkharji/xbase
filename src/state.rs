#[cfg(feature = "daemon")]
use crate::Result;

#[cfg(feature = "daemon")]
use std::path::PathBuf;

#[cfg(feature = "daemon")]
use tap::Pipe;

/// Build Server State.
#[derive(Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct State {
    /// Managed Workspaces
    #[cfg(feature = "daemon")]
    pub projects: crate::store::ProjectStore,
    /// Managed Clients
    #[cfg(feature = "daemon")]
    pub clients: crate::store::ClientStore,
    /// Managed watchers
    #[cfg(feature = "daemon")]
    pub watcher: crate::store::WatchStore,
    /// Managed runners
    // #[cfg(feature = "daemon")]
    // pub runners: crate::store::WatchStore,
    #[cfg(feature = "daemon")]
    pub devices: crate::store::Devices,
}

#[cfg(feature = "daemon")]
impl State {
    /// Get all projects that client have access to it.
    #[allow(dead_code)]
    async fn get_client_projects<'a>(
        &'a self,
        client: &'a crate::client::Client,
    ) -> Result<Vec<(&'a PathBuf, &'a crate::types::Project)>> {
        self.projects
            .iter()
            .filter(|(_, p)| p.clients.contains(&client.pid))
            .collect::<Vec<(&'a PathBuf, &'a crate::types::Project)>>()
            .pipe(Ok)
    }

    #[allow(dead_code)]
    async fn get_clients_with_project_root(
        &self,
        root: &PathBuf,
    ) -> Result<Vec<(&i32, &crate::nvim::NvimClient)>> {
        self.clients
            .iter()
            .filter(|(_, nvim)| nvim.roots.contains(root))
            .collect::<Vec<(&i32, &crate::nvim::NvimClient)>>()
            .pipe(Ok)
    }

    pub fn try_into_string(&self) -> Result<String> {
        Ok(serde_json::to_string(&self)?)
    }

    pub async fn sync_client_state(&self) -> Result<()> {
        let state_str = self.try_into_string()?;
        let update_state_script = format!("vim.g.xbase= vim.json.decode([[{state_str}]])");
        tracing::info!("Syncing state to all nvim instance");

        self.clients.update_state(&update_state_script).await?;

        Ok(())
    }

    pub async fn validate(&mut self) {
        let mut invalid_pids = vec![];

        self.clients.retain(|pid, _| {
            crate::util::pid::exists(pid, || {
                tracing::error!("{pid} no longer valid");
                invalid_pids.push(*pid);
            })
        });

        if !invalid_pids.is_empty() {
            for pid in invalid_pids.iter() {
                self.projects.iter_mut().for_each(|(_, p)| {
                    p.clients.retain(|client_pid| pid != client_pid);
                });
            }
            self.projects.retain(|_, p| !p.clients.is_empty())
        }
    }
}
