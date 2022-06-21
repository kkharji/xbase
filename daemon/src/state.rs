use crate::Result;

/// Build Server State.
#[derive(Default, Debug, serde::Serialize)]
pub struct State {
    /// Managed Workspaces
    pub projects: crate::store::ProjectStore,
    /// Managed Clients
    pub clients: crate::store::ClientStore,
    /// Managed watchers
    pub watcher: crate::store::WatchStore,
    /// Available Devices
    pub devices: crate::store::Devices,
}

impl State {
    pub fn try_into_string(&self) -> Result<String> {
        Ok(serde_json::to_string(&self)?)
    }

    pub async fn sync_client_state(&self) -> Result<()> {
        let state_str = self.try_into_string()?;
        let update_state_script = format!("vim.g.xbase= vim.json.decode([[{state_str}]])");
        log::info!("Syncing state to all nvim instance");

        self.clients.update_state(&update_state_script).await?;

        Ok(())
    }

    pub async fn validate(&mut self) {
        let mut invalid_pids = vec![];

        self.clients.retain(|pid, _| {
            crate::util::pid::exists(pid, || {
                log::error!("{pid} no longer valid");
                invalid_pids.push(*pid);
            })
        });

        if !invalid_pids.is_empty() {
            for pid in invalid_pids.iter() {
                self.projects.iter_mut().for_each(|(_, p)| {
                    p.clients_mut().retain(|client_pid| pid != client_pid);
                });
            }
            self.projects.retain(|_, p| !p.clients().is_empty())
        }
    }
}
