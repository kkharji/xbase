use crate::state::SharedState;
use crate::Command;
use notify::{Error, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::result::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
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
async fn should_ignore(last_seen: Arc<Mutex<String>>, path: &str) -> bool {
    // HACK: Always return false for project.yml
    let path = path.to_string();
    if path.contains("project.yml") {
        return false;
    }
    let mut last_seen = last_seen.lock().await;
    if last_seen.to_string() == path {
        return true;
    } else {
        *last_seen = path;
        return false;
    }
}

// TODO: Cleanup get_ignore_patterns and decrease duplications
async fn get_ignore_patterns(state: SharedState, root: &String) -> Vec<String> {
    let mut patterns: Vec<String> = vec![
        "**/.git/**",
        "**/*.xcodeproj/**",
        "**/.*",
        "**/build/**",
        "**/buildServer.json",
    ]
    .iter()
    .map(|e| e.to_string())
    .collect();

    // FIXME: Addding extra ignore patterns to `ignore` local config requires restarting deamon.
    let extra_patterns = state
        .lock()
        .await
        .workspaces
        .get(root)
        .unwrap()
        .get_ignore_patterns();

    if let Some(extra_patterns) = extra_patterns {
        patterns.extend(extra_patterns);
    }

    patterns
}

fn new(state: SharedState, root: String) -> tokio::task::JoinHandle<anyhow::Result<()>> {
    // NOTE: should watch for registerd directories?
    // TODO: Support provideing additional ignore wildcard
    //
    // Some files can be generated as direct result of running build command.
    // In my case this `Info.plist`.
    //
    // For example,  define key inside project.yml under xcodebase key, ignoreGlob of type array.

    tokio::spawn(async move {
        let (tx, mut rx) = mpsc::channel(100);

        let mut watcher = RecommendedWatcher::new(move |res: Result<Event, Error>| {
            if res.is_ok() {
                tx.blocking_send(res.unwrap()).unwrap()
            };
        })?;

        watcher.watch(Path::new(&root), RecursiveMode::Recursive)?;
        watcher.configure(notify::Config::NoticeEvents(true))?;

        // HACK: ignore seen paths.
        let last_seen = Arc::new(Mutex::new(String::default()));

        // HACK: convert back to Vec<&str> for Glob to work.
        let patterns = get_ignore_patterns(state.clone(), &root).await;
        let patterns = patterns.iter().map(AsRef::as_ref).collect::<Vec<&str>>();
        let ignore = wax::any::<Glob, _>(patterns).unwrap();

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

            if ignore.is_match(&*path_string) {
                continue;
            }

            // debug!("[FSEVENT] {:?}", &event);
            // NOTE: maybe better handle in tokio::spawn?
            match &event.kind {
                notify::EventKind::Create(_) => {
                    tokio::time::sleep(Duration::new(1, 0)).await;
                    debug!("[FileCreated]: {:?}", path);
                }
                notify::EventKind::Remove(_) => {
                    tokio::time::sleep(Duration::new(1, 0)).await;
                    debug!("[FileRemoved]: {:?}", path);
                }
                notify::EventKind::Modify(m) => {
                    match m {
                        notify::event::ModifyKind::Data(e) => match e {
                            notify::event::DataChange::Content => {
                                if !path_string.contains("project.yml") {
                                    continue;
                                }
                                tokio::time::sleep(Duration::new(1, 0)).await;
                                debug!("[XcodeGenConfigUpdate]");
                                // HACK: Not sure why, but this is needed because xcodegen break.
                            }
                            _ => continue,
                        },
                        notify::event::ModifyKind::Name(_) => {
                            // HACK: only account for new path and skip duplications
                            if !Path::new(&path).exists()
                                || should_ignore(last_seen.clone(), &path_string).await
                            {
                                continue;
                            }
                            tokio::time::sleep(Duration::new(1, 0)).await;
                            debug!("[FileRenamed]: {:?}", path);
                        }
                        _ => continue,
                    }
                }
                _ => continue,
            }

            trace!("[NewEvent] {:#?}", &event);

            // let mut state = state.lock().await;

            match state.lock().await.workspaces.get_mut(&root) {
                Some(w) => {
                    w.on_dirctory_change(path, event.kind).await?;
                }
                // NOTE: should stop watch here
                None => continue,
            };
        }
        Ok(())
    })
}
