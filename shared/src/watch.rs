use crate::state::SharedState;
use crate::Command;
use notify::{Error, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::result::Result;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tracing::{debug, trace};
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
fn should_ignore(last_seen: Arc<Mutex<String>>, path: &str) -> bool {
    let path = path.to_string();
    let mut last_seen = last_seen.lock().unwrap();
    if last_seen.to_string() == path {
        return true;
    } else {
        *last_seen = path;
        return false;
    }
}

fn new(state: SharedState, root: String) -> tokio::task::JoinHandle<anyhow::Result<()>> {
    // NOTE: should watch for registerd directories?
    // TODO: Support provideing additional ignore wildcard
    //
    // Some files can be generated as direct result of running build command.
    // In my case this `Info.plist`.
    //
    // For example,  define key inside project.yml under xcodebase key, ignoreGlob of type array.
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

        // let gitignore = gitignore::File::new(path::Path::new(&gitignore_path)).unwrap();

        while let Some(event) = rx.recv().await {
            let state = state.clone();
            let path = match event.paths.get(0) {
                Some(p) => p.clone(),
                None => continue,
            };

            let path_string = match path.to_str() {
                Some(s) => s.to_string(),
                None => continue,
            };

            if should_ignore(last_seen.clone(), &path_string) || ignore.is_match(&*path_string) {
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
