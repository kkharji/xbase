use crate::constants::DAEMON_STATE;
use crate::util::log_request;
use crate::Result;
use xbase_proto::DropRequest;

/// handle drop request
pub async fn handle(
    DropRequest {
        client,
        remove_client,
    }: DropRequest,
) -> Result<()> {
    log_request!("Drop", client);

    let state = DAEMON_STATE.clone();
    let ref mut state = state.lock().await;

    if state.clients.contains_key(&client.pid) {
        // NOTE: Should only be Some if no more client depend on it
        if let Some(_) = state.projects.remove(&client).await? {
            // NOTE: Remove project watchers
            state.watcher.remove(&client);
        }

        // NOTE: Try removing client with given pid
        if remove_client {
            state.clients.remove(&client);
        }

        // NOTE: Sink state to all client vim.g.xbase.state
        state.sync_client_state().await?;
    }

    Ok(())
}
