use super::*;

/// Register new client with workspace
#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub client: Client,
}

use crate::constants::DAEMON_STATE;

#[async_trait]
impl Handler for RegisterRequest {
    async fn handle(self) -> Result<()> {
        let Self { client } = &self;

        let state = DAEMON_STATE.clone();
        let ref mut state = state.lock().await;

        client.register_self(state).await?;
        client.register_project(state).await?;

        if client.ensure_server_support(state, None).await? {
            let ref name = client.abbrev_root();
            client.echo_msg(state, name, "setup: ✅").await;
        }

        state.sync_client_state().await?;

        Ok(())
    }
}
