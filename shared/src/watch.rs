use crate::state::SharedState;
use crate::Command;
use notify::{RecursiveMode, Watcher};
use std::path::Path;
use tokio::sync::mpsc;
use tracing::{debug, info, trace};

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

fn new(state: SharedState, root: String) -> tokio::task::JoinHandle<anyhow::Result<()>> {
    tokio::spawn(async move {
        let (tx, mut rx) = mpsc::channel(100);
        // FIXME: Sometimes (more then one/duplicated) event is processed at the same time
        let mut watcher = notify::recommended_watcher(
            move |res: std::result::Result<notify::Event, notify::Error>| {
                if res.is_ok() {
                    tx.blocking_send(res.unwrap()).unwrap()
                };
            },
        )?;

        watcher.watch(Path::new(&root), RecursiveMode::Recursive)?;
        watcher.configure(notify::Config::PreciseEvents(true))?;

        info!("[New::{:?}]", &root);

        while let Some(event) = rx.recv().await {
            let state = state.clone();
            let path = match event.paths.get(0) {
                Some(p) => p.clone(),
                None => continue,
            };

            // NOTE: should watch for registerd directories?
            if path.to_str().unwrap().contains(".git") {
                continue;
            }

            // NOTE: maybe better handle in tokio::spawn?
            match event.kind {
                notify::EventKind::Create(_) => {
                    debug!("[FileCreated]: {:?}", path);
                }
                notify::EventKind::Remove(_) => {
                    debug!("[FileRemoved]: {:?}", path);
                }
                _ => continue,
            }

            trace!("[NewEvent] {:#?}", event);

            let state = state.lock().await;
            let _workspace = match state.workspaces.get(&root) {
                Some(w) => w,
                None => continue,
            };
        }
        Ok(())
    })
}
