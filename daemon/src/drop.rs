use crate::{RequestHandler, Result};
use async_trait::async_trait;
use xbase_proto::DropRequest;

#[async_trait]
impl RequestHandler for DropRequest {
    async fn handle(self) -> Result<()> {
        use crate::constants::DAEMON_STATE;
        let Self { client, .. } = self;

        let (title, sep) = crate::util::handler_log_content("Drop", &client);
        log::info!("{sep}",);
        log::info!("{title}",);
        log::info!("{sep}",);

        let state = DAEMON_STATE.clone();
        let ref mut state = state.lock().await;

        if state.clients.contains_key(&client.pid) {
            // NOTE: Should only be Some if no more client depend on it
            if let Some(_) = state.projects.remove(&client).await? {
                // NOTE: Remove project watchers
                state.watcher.remove(&client);
            }

            // NOTE: Try removing client with given pid
            if self.remove_client {
                state.clients.remove(&client);
            }

            // NOTE: Sink state to all client vim.g.xbase.state
            state.sync_client_state().await?;
        }

        Ok(())
    }
}
