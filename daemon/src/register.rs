use crate::compile;
use crate::constants::DAEMON_STATE;
use crate::Error;
use crate::RequestHandler;
use crate::Result;
use async_trait::async_trait;
use xbase_proto::RegisterRequest;

#[async_trait]
impl RequestHandler for RegisterRequest {
    async fn handle(self) -> Result<()> {
        let Self { client } = &self;

        let state = DAEMON_STATE.clone();
        let ref mut state = state.lock().await;

        if let Ok(project) = state.projects.get_mut(&client.root) {
            project.add_client(client.pid);
        } else {
            state.projects.add(client).await?;
            let ignore_pattern = state
                .projects
                .get(&client.root)
                .unwrap()
                .watchignore()
                .clone();

            state.watcher.add(client, ignore_pattern).await?;
        }
        // NOTE: The following blocks register request due to nvim_rs rpc
        let client = client.clone();
        tokio::spawn(async move {
            let client = &client;
            let state = DAEMON_STATE.clone();
            let ref mut state = state.lock().await;
            state.clients.add(client).await?;

            if compile::ensure_server_support(state, client, None).await? {
                let ref name = client.abbrev_root();
                state
                    .clients
                    .echo_msg(&client.root, name, "setup: ✅")
                    .await;
            }

            state.sync_client_state().await?;

            Ok::<_, Error>(())
        });

        Ok(())
    }
}
