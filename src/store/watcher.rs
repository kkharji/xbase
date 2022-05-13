use crate::daemon::WatchTarget;
use crate::watcher::WatchHandler;

use crate::types::{BuildConfiguration, Client, Root};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct WatchStore {
    projects: HashMap<Root, WatchHandler>,
    targets: HashMap<String, WatchHandler>,
}

impl WatchStore {
    pub async fn add_project_watcher(&mut self, client: &Client, ignore_pattern: Vec<String>) {
        let root = client.root.clone();

        tracing::info!("AddProjectWatcher(root: {})", client.abbrev_root());

        let handler = WatchHandler::new_project_watcher(client.clone(), ignore_pattern);

        self.projects.insert(root, handler);
    }

    pub async fn remove_project_watcher(&mut self, client: &Client) {
        if let Some(handle) = self.projects.get(&client.root) {
            handle.inner().abort();
        };
        tracing::info!("RemoveProjectWatcher({})", client.abbrev_root());

        self.projects.remove(&client.root);
    }
}

impl WatchStore {
    pub async fn add_target_watcher(&mut self, request: &WatchTarget, ignore_pattern: Vec<String>) {
        let key = target_key(&request.config, &request.client);
        let handler = WatchHandler::new_target_watcher(request.clone(), ignore_pattern);

        tracing::info!(
            "AddTargetWatcher(\"{}\", {})",
            request.config,
            request.client.abbrev_root()
        );

        self.targets.insert(key, handler);
    }

    pub async fn remove_target_watcher(&mut self, request: &WatchTarget, client: &Client) {
        let key = target_key(&request.config, &request.client);

        if let Some(handle) = self.targets.get(&key) {
            handle.inner().abort();
        };

        tracing::info!(
            "RemoveTargetWatcher(\"{}\", {})",
            request.config.target,
            client.abbrev_root()
        );

        self.targets.remove(&key);
    }

    pub async fn remove_target_watcher_for_root(&mut self, root: &PathBuf) {
        let root = root.display().to_string();
        self.targets
            .iter()
            .filter(|(key, _)| key.contains(&root))
            .for_each(|(_, handler)| handler.inner().abort());

        self.targets.retain(|key, _| !key.contains(&root));
    }
}

fn target_key(config: &BuildConfiguration, client: &Client) -> String {
    format!("{}:{config}", client.root.display())
}
