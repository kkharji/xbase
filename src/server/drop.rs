use super::*;
use crate::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

/// Drop a given set of roots to be dropped (i.e. unregistered)
#[derive(Debug, Serialize, Deserialize, TypeScriptify)]
pub struct DropRequest {
    pub roots: Vec<PathBuf>,
}

#[async_trait]
impl RequestHandler<()> for DropRequest {
    async fn handle(self) -> Result<()> {
        let DropRequest { roots } = self;
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
                tracing::info!("[{key}] project removed");
                projects.remove(&root);

                if let Some(watcher) = watchers.get(&root) {
                    watcher.lock().await.handler.abort();
                    watchers.remove(&root);
                    tracing::info!("[{key}] watcher removed");
                };

                broadcasters.remove(&root).map(|l| {
                    l.abort();
                });
                tracing::info!("[{}] Dropped", root.as_path().name().unwrap());
            }
        }

        Ok(())
    }
}
