use super::*;

/// Drop a client
#[derive(Debug, Serialize, Deserialize)]
pub struct DropRequest {
    client: Client,
    #[serde(default)]
    remove_client: bool,
}

#[async_trait]
impl Handler for DropRequest {
    async fn handle(self) -> Result<()> {
        use crate::constants::DAEMON_STATE;
        let Self { client, .. } = self;

        let state = DAEMON_STATE.clone();
        let ref mut state = state.lock().await;

        if client.is_registered(state) {
            tracing::info!("Drop({}: {})", client.pid, client.abbrev_root());
            // NOTE: Should only be Some if no more client depend on it
            if let Some(_) = state.projects.remove(&client).await? {
                // NOTE: Remove project watchers
                client.remove_watcher(state);
            }

            // NOTE: Try removing client with given pid
            if self.remove_client {
                client.remove_self(state);
            }

            // NOTE: Sink state to all client vim.g.xbase.state
            state.sync_client_state().await?;
        }

        Ok(())
    }
}
