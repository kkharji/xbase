mod event;
mod interface;
mod state;

use crate::broadcast::Broadcast;
use crate::project::ProjectImplementer;
use crate::store::TryGetDaemonObject;
use crate::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Weak};
use tokio::sync::mpsc::channel;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use xbase_proto::{IntoResult, PathExt};

pub use {event::*, interface::*};

#[derive(derive_deref_rs::Deref)]
pub struct WatchService {
    #[deref]
    pub listeners: HashMap<String, Box<(dyn Watchable + Send + Sync + 'static)>>,
    pub handler: JoinHandle<Result<()>>,
}

impl WatchService {
    pub async fn new(
        root: &PathBuf,
        watchignore: Vec<String>,
        broadcast: Weak<Broadcast>,
        project: Weak<Mutex<ProjectImplementer>>,
    ) -> Result<Self> {
        let root = root.clone();
        let mut discards = vec![];
        let internal_state = Default::default();
        let listeners = Default::default();

        let handler = tokio::spawn(async move {
            let (mut rx, _w) = start_watcher(&root)?;
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
                        break;
                    }
                };

                let project = &mut match project.upgrade() {
                    Some(p) => p.lock_owned().await,
                    None => {
                        let msg = format!(r"No project found for {root:?}, dropping watcher ..");
                        broadcast.warn(msg);

                        break;
                    }
                };

                if event.is_create_event()
                    || event.is_remove_event()
                    || event.is_content_update_event()
                    || event.is_rename_event() && !event.is_seen()
                {
                    project
                        .ensure_server_support(event.into(), &broadcast)
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
                        } else {
                            discards.push(key.to_string());
                        }
                        continue;
                    }

                    if listener.should_trigger(event).await {
                        // WARN: This would block if trigger it self access watcher
                        // Use for trigger inner handle like run service handler
                        let trigger =
                            listener.trigger(project, event, &broadcast, Arc::downgrade(&w));

                        if let Err(err) = trigger.await {
                            log::error!("trigger errored for `{key}`!: {err}");
                        }
                    }
                }

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
}

fn start_watcher(
    root: &PathBuf,
) -> Result<(
    tokio::sync::mpsc::Receiver<notify::Event>,
    impl notify::Watcher,
)> {
    use notify::{Config, RecommendedWatcher, RecursiveMode::Recursive, Watcher};
    let (tx, rx) = channel::<notify::Event>(1);
    let create = <RecommendedWatcher as Watcher>::new;
    let to_err = |e: notify::Error| crate::Error::Unexpected(e.to_string());

    let mut watcher = create(move |res: notify::Result<notify::Event>| {
        res.map(|event| tx.blocking_send(event).unwrap()).ok();
    })
    .map_err(to_err)?;

    watcher.watch(root, Recursive).map_err(to_err)?;
    watcher
        .configure(Config::NoticeEvents(true))
        .map_err(to_err)?;

    Ok((rx, watcher))
}
