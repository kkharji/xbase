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
        let (title, sep) = crate::util::handler_log_content("Register", &client);
        log::info!("{sep}");
        log::info!("{title}");
        log::info!("{sep}");

        let state = DAEMON_STATE.clone();
        let ref mut state = state.lock().await;

        if let Ok(project) = state.projects.get_mut(&client.root) {
            project.add_client(client.pid);
        } else {
            state.projects.add(client).await?;
            let project = state.projects.get(&client.root).unwrap();
            let watchignore = project.watchignore().clone();
            let name = project.name().to_string();

            state.watcher.add(client, watchignore, &name).await?;
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
