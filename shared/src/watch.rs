use crate::state::SharedState;
use crate::Command;
use notify::{Error, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::result::Result;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, info, trace};
use wax::{Glob, Pattern};

// TODO: Stop handle

pub async fn update(state: SharedState, _msg: Command) {
    let copy = state.clone();
    let mut current_state = copy.lock().await;
    let mut watched_roots: Vec<String> = vec![];
    let mut start_watching: Vec<String> = vec![];

    // TODO: Remove wathcers for workspaces that are no longer exist

    for key in current_state.watchers.keys() {
        watched_roots.push(key.clone());
    }

    for key in current_state.workspaces.keys() {
        if !watched_roots.contains(key) {
            start_watching.push(key.clone());
        }
    }

    for root in start_watching {
        let handle = new(state.clone(), root.clone());
        current_state.watchers.insert(root, handle);
    }
}

/// HACK: ignore seen paths.
///
/// Sometiems we get event for the same path, particularly
/// `ModifyKind::Name::Any` is ommited twice for the new path
/// and once for the old path.
///
/// This will compare last_seen with path, updates `last_seen` if not match,
/// else returns true.
async fn should_ignore(last_seen: Arc<Mutex<String>>, path: &str) -> bool {
    let path = path.to_string();
    let mut last_seen = last_seen.lock().await;
    if last_seen.to_string() == path {
        return true;
    } else {
        *last_seen = path;
        return false;
    }
}

fn new(state: SharedState, root: String) -> tokio::task::JoinHandle<anyhow::Result<()>> {
    // NOTE: should watch for registerd directories?
    let ignore = wax::any::<Glob, _>(["**/.git/**", "**/*.xcodeproj/**", "**/.*"]).unwrap();

    tokio::spawn(async move {
        let (tx, mut rx) = mpsc::channel(100);

        let mut watcher = RecommendedWatcher::new(move |res: Result<Event, Error>| {
            if res.is_ok() {
                tx.blocking_send(res.unwrap()).unwrap()
            };
        })?;

        watcher.watch(Path::new(&root), RecursiveMode::Recursive)?;

        // HACK: ignore seen paths.
        let last_seen = Arc::new(Mutex::new(String::default()));

        while let Some(event) = rx.recv().await {
            let state = state.clone();
            let path = match event.paths.get(0) {
                Some(p) => match p.clone().to_str() {
                    Some(s) => s.to_string(),
                    None => continue,
                },
                None => continue,
            };

            if should_ignore(last_seen.clone(), &path).await || ignore.is_match(&*path) {
                continue;
            }

            // NOTE: maybe better handle in tokio::spawn?
            match &event.kind {
                notify::EventKind::Create(_) => {
                    debug!("[FileCreated]: {:?}", path);
                }
                notify::EventKind::Remove(_) => {
                    debug!("[FileRemoved]: {:?}", path);
                }
                notify::EventKind::Modify(m) => match m {
                    notify::event::ModifyKind::Name(_) => {
                        // HACK: only account for new path
                        if !Path::new(&path).exists() {
                            continue;
                        }
                        debug!("[FileRenamed]: NewName: {:?}", path);
                    }
                    _ => continue,
                },
                _ => continue,
            }

            trace!("[NewEvent] {:#?}", &event);

            let state = state.lock().await;
            let workspace = match state.workspaces.get(&root) {
                Some(w) => w,
                None => continue,
            };

            workspace.on_dirctory_change(path, event.kind).await?;
        }
        Ok(())
    })
}
