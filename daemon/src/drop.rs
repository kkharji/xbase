use crate::constants::DAEMON_STATE;
use crate::util::log_request;
use crate::Result;
use xbase_proto::Client;

/// handle drop request
pub async fn handle(Client { root, .. }: Client) -> Result<()> {
    let state = DAEMON_STATE.clone();
    let ref mut state = state.lock().await;

    // TODO: warn
    if !state.projects.contains_key(&root) {
        return Ok(());
    }

    log_request!("Drop", root);

    // NOTE: Should only be Some if no more client depend on it
    if let Some(_) = state.projects.remove(&root).await? {
        state.watcher.remove(&root)?;
        state.broadcasters.remove(&root)
    }

    Ok(())
}
