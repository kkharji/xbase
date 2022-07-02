mod event;
pub use event::{Event, EventKind};

use crate::broadcast::Broadcast;
use crate::project::ProjectImplementer;
use crate::store::TryGetDaemonObject;
use crate::Result;
use async_trait::async_trait;
use notify::{Config, RecommendedWatcher, RecursiveMode::Recursive, Watcher};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex as BMutex, Weak};
use std::time::SystemTime;
use tokio::sync::mpsc::channel;
use tokio::sync::{Mutex, OwnedMutexGuard};
use tokio::task::JoinHandle;
use xbase_proto::{IntoResult, PathExt};

#[derive(derive_deref_rs::Deref)]
pub struct WatchService {
    #[deref]
    pub listeners: HashMap<String, Box<(dyn Watchable + Send + Sync + 'static)>>,
    pub handler: JoinHandle<Result<()>>,
}

pub struct InternalState {
    debounce: Arc<BMutex<SystemTime>>,
    last_path: Arc<BMutex<PathBuf>>,
}

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
    pub async fn new(
        root: &PathBuf,
        watchignore: Vec<String>,
        broadcast: Weak<Broadcast>,
        project: Weak<Mutex<ProjectImplementer>>,
    ) -> Result<Self> {
        let root = root.clone();
        let listeners = Default::default();

        let handler = tokio::spawn(async move {
            let mut discards = vec![];
            let internal_state = InternalState::default();

            let (tx, mut rx) = channel::<notify::Event>(1);
            let mut w = <RecommendedWatcher as Watcher>::new(move |res| {
                if let Ok(event) = res {
                    tx.blocking_send(event).unwrap()
                }
            })
            .map_err(|e| crate::Error::Unexpected(e.to_string()))?;
            w.watch(&root, Recursive)
                .map_err(|e| crate::Error::Unexpected(e.to_string()))?;
            w.configure(Config::NoticeEvents(true))
                .map_err(|e| crate::Error::Unexpected(e.to_string()))?;

            let watchignore = watchignore.iter().map(AsRef::as_ref).collect::<Vec<&str>>();

            let ignore = wax::any::<wax::Glob, _>(watchignore).unwrap();

            while let Some(event) = rx.recv().await {
                let ref event = match Event::new(&ignore, &internal_state, event) {
                    Some(e) => e,
                    None => continue,
                };

                // IGNORE EVENTS OF RENAME FOR PATHS THAT NO LONGER EXISTS
                if !event.path().exists() && event.is_rename_event() {
                    log::debug!("{} [ignored]", event);
                    continue;
                }

                let broadcast = match broadcast.upgrade() {
                    Some(broadcast) => broadcast,
                    None => {
                        log::warn!(r"No broadcast found for {root:?}, dropping watcher ..");
                        return Ok(());
                    }
                };

                let mut project = match project.upgrade() {
                    Some(p) => p.lock_owned().await,
                    None => {
                        broadcast.warn(format!(
                            r"No project found for {root:?}, dropping watcher .."
                        ));

                        return Ok(());
                    }
                };

                if event.is_create_event()
                    || event.is_remove_event()
                    || event.is_content_update_event()
                    || event.is_rename_event() && !event.is_seen()
                {
                    project
                        .ensure_server_support(Some(event), &broadcast)
                        .await?;
                };

                let w = match root.try_get_mutex_watcher().await {
                    Ok(w) => w,
                    Err(err) => {
                        log::error!(r#"Unable to get watcher for {root:?}: {err}"#);
                        log::info!(r#"Dropping watcher for {root:?}: {err}"#);
                        break;
                    }
                };

                let mut watcher = w.lock().await;

                for (key, listener) in watcher.listeners.iter() {
                    if listener.should_discard(event).await {
                        if let Err(err) = listener.discard().await {
                            log::error!(" discard errored for `{key}`!: {err}");
                        }
                        discards.push(key.to_string());
                    } else if listener.should_trigger(event).await {
                        // WARN: This would block if trigger need to access w
                        let trigger =
                            listener.trigger(&mut project, event, &broadcast, Arc::downgrade(&w));

                        if let Err(err) = trigger.await {
                            log::error!("trigger errored for `{key}`!: {err}");
                        }
                    }
                }
                // let watcher = state.watcher.get_mut(&root).unwrap();

                for key in discards.iter() {
                    log::info!("[{key:?}] discarded");
                    watcher.listeners.remove(key);
                }

                discards.clear();
                internal_state.update_debounce();

                log::info!("{event} consumed successfully");
            }

            log::info!("Dropped {:?}!!", root.as_path().abbrv()?.display());

            Ok(())
        });

        Ok(Self { handler, listeners })
    }

    pub fn add<W: Watchable>(&mut self, watchable: W) -> Result<()> {
        let key = watchable.to_string();
        log::info!(r#"Add: {key:?}"#);

        let other = self.listeners.insert(key, Box::new(watchable));
        if let Some(watchable) = other {
            let key = watchable.to_string();
            log::error!("Watchable with `{key}` already exists!")
        }

        Ok(())
    }

    pub fn remove(&mut self, key: &String) -> Result<Box<dyn Watchable>> {
        log::info!("Remove: `{key}`");
        let item = self.listeners.remove(key).into_result("Watchable", key)?;
        Ok(item)
    }
}

impl Default for InternalState {
    fn default() -> Self {
        Self {
            debounce: Arc::new(BMutex::new(SystemTime::now())),
            last_path: Default::default(),
        }
    }
}

impl InternalState {
    pub fn update_debounce(&self) {
        let mut debounce = self.debounce.lock().unwrap();
        *debounce = SystemTime::now();
        log::trace!("Debounce updated!!!");
    }

    pub fn last_run(&self) -> u128 {
        self.debounce.lock().unwrap().elapsed().unwrap().as_millis()
    }

    /// Get a reference to the internal state's last path.
    #[must_use]
    pub fn last_path(&self) -> Arc<BMutex<PathBuf>> {
        self.last_path.clone()
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
