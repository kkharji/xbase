use crate::constants::DAEMON_STATE;
use crate::Result;
use std::path::PathBuf;
use xbase_proto::PathExt;

/// handle drop request
pub async fn handle(root: PathBuf) -> Result<()> {
    let state = DAEMON_STATE.clone();
    let ref mut state = state.lock().await;

    // TODO: warn
    if !state.projects.contains_key(&root) {
        return Ok(());
    }

    // NOTE: Should only be Some if no more client depend on it
    if let Some(_) = state.projects.remove(&root).await? {
        state.watcher.remove(&root)?;
        state.broadcasters.remove(&root);
        log::info!("dropped {}", root.as_path().name().unwrap());
    }

    Ok(())
}
