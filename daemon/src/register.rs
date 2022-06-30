use crate::compile;
use crate::constants::DAEMON_STATE;
use crate::Result;
use std::path::PathBuf;
use xbase_proto::Client;
use xbase_proto::PathExt;
use xbase_proto::OK;

/// Handle RegisterRequest
pub async fn handle(Client { root, .. }: Client) -> Result<PathBuf> {
    log::info!("Register {}", root.as_path().name().unwrap());

    let state = DAEMON_STATE.clone();
    let ref mut state = state.lock().await;
    let broadcast = state.broadcasters.get_or_init(&root).await?;
    let broadcast = broadcast.upgrade().unwrap();
    let logger_path = broadcast.address().clone();

    drop(state);

    tokio::spawn(async move {
        let state = DAEMON_STATE.clone();
        let ref mut state = state.lock().await;
        let name: String;

        if let Ok(project) = state.projects.get_mut(&root) {
            project.inc_clients();
        } else {
            state.projects.register(&root, &broadcast).await?;
            let project = state.projects.get(&root).unwrap();
            let watchignore = project.watchignore().clone();
            name = project.name().to_string();

            state
                .watcher
                .add(&root, watchignore, &name, &broadcast)
                .await?;
        }
        compile::ensure_server_support(state, &root, None, &broadcast).await?;

        OK(())
    });

    Ok(logger_path)
}
