use crate::broadcast::Broadcast;
use crate::watch::WatchService;
use crate::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use xbase_proto::IntoResult;

use crate::util::fs::{self, PathExt};
#[derive(Default, Debug)]
pub struct WatchStore(HashMap<PathBuf, WatchService>);

impl WatchStore {
    pub async fn add(
        &mut self,
        root: &PathBuf,
        watchignore: Vec<String>,
        name: &str,
        logger: &Arc<Broadcast>,
    ) -> Result<()> {
        let handler = WatchService::new(root.into(), watchignore, Arc::downgrade(&logger)).await?;
        log::info!("[{}] added", name);
        self.0.insert(root.clone(), handler);
        Ok(())
    }

    pub fn remove(&mut self, root: &PathBuf) -> Result<()> {
        if let Some(handle) = self.0.get(root) {
            handle.handler.abort();
        };

        log::info!("[{}] removed", root.as_path().abbrv()?.display());

        self.0.remove(root);

        Ok(())
    }

    pub fn get(&self, root: &PathBuf) -> Result<&WatchService> {
        let watcher = self.0.get(root).into_result("Watcher", root)?;
        log::trace!("[{}] accessed", fs::abbrv_path(root));
        Ok(watcher)
    }

    pub fn get_mut(&mut self, root: &PathBuf) -> Result<&mut WatchService> {
        let watcher = self.0.get_mut(root).into_result("Watcher", root)?;
        log::trace!("[{}] accessed", fs::abbrv_path(root));
        Ok(watcher)
    }
}
