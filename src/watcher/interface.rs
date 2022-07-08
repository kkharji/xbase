use super::*;
use crate::*;
use async_trait::async_trait;
use std::sync::{Arc, Weak};
use tokio::sync::{Mutex, OwnedMutexGuard};

/// Trait to make an object react to filesystem changes.
///
/// ToString is required in order to store watchable in HashMap
#[async_trait]
pub trait Watchable: ToString + Send + Sync + 'static {
    /// Trigger Restart of Watchable.
    async fn trigger(
        &self,
        project: &mut OwnedMutexGuard<ProjectImplementer>,
        event: &Event,
        broadcast: &Arc<Broadcast>,
        watcher: Weak<Mutex<WatchService>>,
    ) -> Result<()>;

    /// A function that controls whether a a Watchable should restart
    async fn should_trigger(&self, event: &Event) -> bool;

    /// A function that controls whether a watchable should be dropped
    async fn should_discard(&self, event: &Event) -> bool;

    /// Drop watchable for watching a given file system
    async fn discard(&self) -> Result<()>;
}

impl WatchService {
    pub fn add<W: Watchable>(&mut self, watchable: W) -> Result<()> {
        let key = watchable.to_string();
        tracing::info!(r#"Add: {key:?}"#);

        let other = self.listeners.insert(key, Box::new(watchable));
        if let Some(watchable) = other {
            let key = watchable.to_string();
            tracing::error!("Watchable with `{key}` already exists!")
        }

        Ok(())
    }

    pub fn remove(&mut self, key: &String) -> Result<Box<dyn Watchable>> {
        tracing::info!("Remove: `{key}`");
        let item = self.listeners.remove(key).into_result("Watchable", key)?;
        Ok(item)
    }
}

impl std::fmt::Debug for WatchService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let listners = self
            .listeners
            .iter()
            .map(|(key, _)| key.to_string())
            .collect::<Vec<String>>();

        f.debug_struct("WatchService")
            .field("listners", &listners)
            .field("handler", &self.handler)
            .finish()
    }
}
