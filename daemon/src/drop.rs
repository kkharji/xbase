use crate::store::{broadcasters, projects, watchers};
use crate::Result;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use xbase_proto::PathExt;

/// handle drop request
pub async fn handle(roots: HashSet<PathBuf>) -> Result<()> {
    let mut watchers = watchers().await;
    let mut broadcasters = broadcasters().await;
    let mut projects = projects().await;

    for root in roots.into_iter() {
        let mut project = if let Some(project) = projects.get(&root).map(Arc::clone) {
            project.lock_owned().await
        } else {
            continue;
        };

        // Remove client pid from project.
        project.dec_clients();

        // Remove project only when no more client using that data.
        if project.clients() == &0 {
            let key = root.as_path().abbrv()?.display();
            log::info!("[{key}] project removed");
            projects.remove(&root);

            if let Some(watcher) = watchers.get(&root) {
                watcher.lock().await.handler.abort();
                watchers.remove(&root);
                log::info!("[{key}] watcher removed");
            };

            broadcasters.remove(&root).map(|l| {
                l.abort();
            });
            log::info!("dropped {}", root.as_path().name().unwrap());
        }
    }

    Ok(())
}
