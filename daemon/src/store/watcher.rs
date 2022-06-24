use crate::watch::WatchService;
use crate::Result;
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use xbase_proto::{Client, IntoResult};

use crate::util::fs;
#[derive(Default, Debug, Serialize)]
pub struct WatchStore(HashMap<PathBuf, WatchService>);

impl WatchStore {
    pub async fn add(
        &mut self,
        client: &Client,
        watchignore: Vec<String>,
        name: &str,
    ) -> Result<()> {
        let handler = WatchService::new(client.to_owned(), watchignore).await?;
        log::info!("[{}] added", name);
        self.0.insert(client.root.clone(), handler);
        Ok(())
    }

    pub fn remove(&mut self, client: &Client) {
        if let Some(handle) = self.0.get(&client.root) {
            handle.handler.abort();
        };

        log::info!("[{}] removed", client.abbrev_root());

        self.0.remove(&client.root);
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
