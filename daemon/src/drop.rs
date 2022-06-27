use crate::constants::DAEMON_STATE;
use crate::util::log_request;
use crate::Result;
use xbase_proto::DropRequest;

/// handle drop request
pub async fn handle(DropRequest { client, .. }: DropRequest) -> Result<()> {
    log_request!("Drop", client);

    let state = DAEMON_STATE.clone();
    let ref mut state = state.lock().await;

    // NOTE: Should only be Some if no more client depend on it
    if let Some(_) = state.projects.remove(&client).await? {
        // NOTE: Remove project watchers
        state.watcher.remove(&client);
        state.loggers.remove_all_by_project_root(&client.root)
    }

    Ok(())
}
