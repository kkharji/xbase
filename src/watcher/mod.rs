mod event;

use crate::*;
use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;
use std::{sync::Mutex, time::SystemTime};
use tokio::sync::mpsc::{self, channel, Receiver};
use tokio::sync::Notify;
use tracing::{error, info, instrument, warn};

pub use event::*;

pub struct Watcher {
    name: String,
    state: WatcherState,
    sender: mpsc::UnboundedSender<runtime::PRMessage>,
    ignore: Vec<String>,
    abort: Arc<Notify>,
    root: PathBuf,
}

impl Watcher {
    pub fn new(
        name: &String,
        state: &WatcherState,
        sender: &mpsc::UnboundedSender<runtime::PRMessage>,
        abort: &Arc<Notify>,
        root: &PathBuf,
        ignore: &Vec<String>,
    ) -> Self {
        Self {
            name: name.clone(),
            state: state.clone(),
            sender: sender.clone(),
            ignore: ignore.clone(),
            abort: abort.clone(),
            root: root.clone(),
        }
    }

    #[instrument(parent = None, name = "FSWatcher", skip_all, fields(name = self.name))]
    pub async fn start(self) {
        let (mut rx, _w) = self.get_watcher().unwrap();
        let watchignore = self.ignore.iter().map(AsRef::as_ref).collect::<Vec<&str>>();
        let ignore = wax::any::<wax::Glob, _>(watchignore).unwrap();

        tracing::info!("Watching");
        loop {
            tokio::select! {
                _ = self.abort.notified() => break,
                event = rx.recv() => {
                    if event.is_none() { break; }
                    let event = event.unwrap();
                    let event = match Event::new(&ignore, &self.state, event) {
                        Some(e) => e,
                        None => continue,
                    };

                    // IGNORE EVENTS OF RENAME FOR PATHS THAT NO LONGER EXISTS
                    if !event.path().exists() && event.is_rename_event() {
                        tracing::debug!("{} [ignored]", event);
                        continue;
                    }
                    self.sender.send(PRMessage::FSEvent(event)).ok();
                }
            }
        }

        tracing::info!("[Dropped]");
    }

    fn get_watcher(&self) -> Result<(Receiver<notify::Event>, impl notify::Watcher)> {
        use notify::{Config, RecommendedWatcher, RecursiveMode::Recursive, Watcher};
        let (tx, rx) = channel::<notify::Event>(1);
        let create = <RecommendedWatcher as Watcher>::new;
        let to_err = |e: notify::Error| crate::Error::Unexpected(e.to_string());

        let mut watcher = create(move |res: notify::Result<notify::Event>| {
            res.map(|event| tx.blocking_send(event).unwrap()).ok();
        })
        .map_err(to_err)?;

        watcher.watch(&self.root, Recursive).map_err(to_err)?;
        watcher
            .configure(Config::NoticeEvents(true))
            .map_err(to_err)?;

        Ok((rx, watcher))
    }
}

/// Trait to make an object react to filesystem changes.
#[async_trait]
pub trait Watchable: ToString + Send + Sync + 'static {
    /// Trigger Restart of Watchable.
    async fn trigger(
        &self,
        project: &mut ProjectImpl,
        ev: &Event,
        b: &Arc<Broadcast>,
    ) -> Result<()>;

    /// A function that controls whether a a Watchable should restart
    async fn should_trigger(&self, ev: &Event) -> bool;

    /// A function that controls whether a watchable should be dropped
    async fn should_discard(&self, ev: &Event) -> bool;

    /// Drop watchable for watching a given file system
    async fn discard(&self);
}

#[derive(Default)]
pub struct WatchSubscribers {
    name: String,
    inner: HashMap<String, Box<(dyn Watchable + Send + Sync + 'static)>>,
}

impl WatchSubscribers {
    pub fn new(name: &String) -> Self {
        Self {
            name: name.clone(),
            inner: Default::default(),
        }
    }
    #[instrument(parent = None, name = "FSWatcher", skip_all, fields(name = self.name))]
    pub fn add<W: Watchable>(&mut self, watchable: W) {
        let key = watchable.to_string();
        if self.inner.contains_key(&key) {
            warn!("trying to add {key}!!");
        } else {
            self.inner.insert(key, Box::new(watchable));
        }
    }

    #[instrument(parent = None, name = "FSWatcher", skip_all, fields(name = self.name))]
    pub async fn remove<S: ToString>(&mut self, t: &S) {
        let key = t.to_string();
        if let Some(w) = self.inner.remove(&key) {
            w.discard().await;
            info!("Removed watch subscriber: `{key}`");
        } else {
            error!("Trying to remove non-existent watch subscriber: `{key}`")
        }
    }

    pub fn keys(&self) -> Vec<String> {
        self.inner.keys().map(ToString::to_string).collect()
    }

    #[instrument(parent = None, name = "FSWatcher", skip_all, fields(name = self.name))]
    pub async fn trigger(
        &mut self,
        project: &mut ProjectImpl,
        event: &Event,
        broadcast: &Arc<Broadcast>,
    ) {
        let mut discards = vec![];

        for (key, w) in self.inner.iter() {
            if w.should_discard(&event).await {
                w.discard().await;
                discards.push(key.to_string());
            } else if w.should_trigger(&event).await {
                let trigger = w.trigger(project, event, broadcast);
                if let Err(err) = trigger.await {
                    error!("trigger errored for `{key}`!: {err}");
                }
            }
        }

        for key in discards {
            info!("Discarded: `{key}`");
            self.inner.remove(&key);
        }
    }
}

#[derive(Clone)]
pub struct WatcherState {
    debounce: Arc<Mutex<SystemTime>>,
    last_path: Arc<Mutex<PathBuf>>,
}

impl WatcherState {
    pub fn new() -> Self {
        Self {
            debounce: Arc::new(Mutex::new(SystemTime::now())),
            last_path: Default::default(),
        }
    }
    pub fn update_debounce(&self) {
        let mut debounce = self.debounce.lock().unwrap();
        *debounce = SystemTime::now();
        tracing::trace!("Debounce updated!!!");
    }

    pub fn last_run(&self) -> u128 {
        self.debounce.lock().unwrap().elapsed().unwrap().as_millis()
    }

    /// Get a reference to the internal state's last path.
    #[must_use]
    pub fn last_path(&self) -> Arc<Mutex<PathBuf>> {
        self.last_path.clone()
    }
}
