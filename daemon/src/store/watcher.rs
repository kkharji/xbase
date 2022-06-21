use crate::error::EnsureOptional;
use crate::watch::WatchService;
use crate::Result;
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use xbase_proto::Client;

#[derive(Default, Debug, Serialize)]
pub struct WatchStore(HashMap<PathBuf, WatchService>);

impl WatchStore {
    pub async fn add(&mut self, client: &Client, ignore_pattern: Vec<String>) -> Result<()> {
        log::info!("[Watcher] add(\"{}\")", client.abbrev_root());
        let handler = WatchService::new(client.to_owned(), ignore_pattern).await?;
        self.0.insert(client.root.clone(), handler);
        Ok(())
    }

    pub fn remove(&mut self, client: &Client) {
        if let Some(handle) = self.0.get(&client.root) {
            handle.handler.abort();
        };

        log::info!("[Watcher] remove(\"{}\")", client.abbrev_root());

        self.0.remove(&client.root);
    }

    pub fn get(&self, root: &PathBuf) -> Result<&WatchService> {
        self.0.get(root).to_result("Watcher", root)
    }

    pub fn get_mut(&mut self, root: &PathBuf) -> Result<&mut WatchService> {
        self.0.get_mut(root).to_result("Watcher", root)
    }
}
