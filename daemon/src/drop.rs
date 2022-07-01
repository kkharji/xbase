use crate::constants::DAEMON_STATE;
use crate::Result;
use std::collections::HashSet;
use std::path::PathBuf;
use xbase_proto::PathExt;

/// handle drop request
pub async fn handle(roots: HashSet<PathBuf>) -> Result<()> {
    let state = DAEMON_STATE.clone();
    let ref mut state = state.lock().await;
    for root in roots.into_iter() {
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
    }

    Ok(())
}
