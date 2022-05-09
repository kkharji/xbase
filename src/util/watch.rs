//! Function to watch file system
//!
//! Mainly used for creation/removal of files and editing of xcodegen config.
use crate::daemon::DaemonState;
use anyhow::Result;
use notify::{Error, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::{future::Future, path::PathBuf, sync::Arc, time::SystemTime};
use std::{path::Path, time::Duration};
use tokio::sync::{mpsc, Mutex};
use tokio::task::JoinHandle;
use wax::{Glob, Pattern};

const COMPILE_START_MSG: &str = "echo 'xcodebase: ⚙ Regenerating compilation database ..'";
const COMPILE_SUCC_MESSAGE: &str = "echo 'xcodebase: ✅ Compilation database regenerated.'";

/// HACK: ignore seen paths.
///
/// Sometimes we get event for the same path, particularly
/// `ModifyKind::Name::Any` is omitted twice for the new path
/// and once for the old path.
///
/// This will compare last_seen with path, updates `last_seen` if not match,
/// else returns true.
#[cfg(feature = "async")]
pub async fn should_ignore(last_seen: Arc<Mutex<String>>, path: &str) -> bool {
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
#[cfg(feature = "daemon")]
pub async fn get_ignore_patterns(state: DaemonState, root: &String) -> Vec<String> {
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

    // Note: Add extra ignore patterns to `ignore` local config requires restarting daemon.
    if let Some(ws) = state.lock().await.workspaces.get(root) {
        if let Some(extra_patterns) = ws.get_ignore_patterns() {
            patterns.extend(extra_patterns);
        }
    }

    patterns
}

pub fn new<F, Fut>(
    root: String,
    state: DaemonState,
    event_handler: F,
) -> JoinHandle<anyhow::Result<()>>
where
    F: Fn(DaemonState, String, PathBuf, Event, Arc<Mutex<String>>, Arc<Mutex<SystemTime>>) -> Fut
        + Send
        + 'static,
    Fut: Future<Output = anyhow::Result<bool>> + Send,
{
    use notify::Config::NoticeEvents;
    let debounce = Arc::new(Mutex::new(SystemTime::now()));

    tokio::spawn(async move {
        let (tx, mut rx) = mpsc::channel(1);
        let mut watcher = RecommendedWatcher::new(move |res: Result<Event, Error>| {
            if let Ok(event) = res {
                if let Err(err) = tx.blocking_send(event) {
                    #[cfg(feature = "logging")]
                    tracing::error!("Fail send event {err}");
                };
            } else {
                tracing::error!("Watch Error: {:?}", res);
            };
        })?;

        // NOTE: should watch for registered directories only?
        watcher.watch(Path::new(&root), RecursiveMode::Recursive)?;
        watcher.configure(NoticeEvents(true))?;

        // HACK: ignore seen paths.
        let last_seen = Arc::new(Mutex::new(String::default()));
        // HACK: convert back to Vec<&str> for Glob to work.
        let patterns = get_ignore_patterns(state.clone(), &root).await;
        let patterns = patterns.iter().map(AsRef::as_ref).collect::<Vec<&str>>();
        let ignore = match wax::any::<Glob, _>(patterns) {
            Ok(i) => i,
            Err(err) => {
                anyhow::bail!("Fail to generate ignore glob: {err}")
            }
        };

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
            let _debounce = debounce.clone();

            let last_run = match _debounce.lock().await.elapsed() {
                Ok(time) => time.as_millis(),
                Err(err) => {
                    #[cfg(feature = "logging")]
                    tracing::error!("Fail to get last_run time: {err}");
                    continue;
                }
            };

            if !(last_run > 1) {
                tracing::debug!("{:?}, paths: {:?}", event.kind, &event.paths);
                tracing::trace!("pass_threshold: {last_run}, {:?}", event);
                continue;
            }

            let future = event_handler(
                state,
                root.clone(),
                path,
                event,
                last_seen.clone(),
                debounce.clone(),
            );
            if let Err(e) = future.await {
                tracing::error!("{e}")
            }
        }
        Ok(())
    })
}

pub async fn recompile_handler(
    state: DaemonState,
    root: String,
    path: PathBuf,
    event: Event,
    last_seen: Arc<Mutex<String>>,
    debounce: Arc<Mutex<SystemTime>>,
) -> anyhow::Result<bool> {
    match &event.kind {
        notify::EventKind::Create(_) => {
            tokio::time::sleep(Duration::new(1, 0)).await;
            tracing::debug!("[FileCreated]: {:?}", path);
        }
        notify::EventKind::Remove(_) => {
            tokio::time::sleep(Duration::new(1, 0)).await;
            tracing::debug!("[FileRemoved]: {:?}", path);
        }
        notify::EventKind::Modify(m) => match m {
            notify::event::ModifyKind::Data(e) => match e {
                notify::event::DataChange::Content => {
                    if !path.display().to_string().contains("project.yml") {
                        return Ok(false);
                    }
                    tokio::time::sleep(Duration::new(1, 0)).await;
                    tracing::debug!("[XcodeGenConfigUpdate]");
                    // HACK: Not sure why, but this is needed because xcodegen break.
                }
                _ => return Ok(false),
            },
            notify::event::ModifyKind::Name(_) => {
                let path_string = path.to_string_lossy();
                // HACK: only account for new path and skip duplications
                if !path.exists() || should_ignore(last_seen.clone(), &path_string).await {
                    return Ok(false);
                }
                tokio::time::sleep(Duration::new(1, 0)).await;
                #[cfg(feature = "logging")]
                tracing::debug!("[FileRenamed]: {:?}", path);
            }
            _ => return Ok(false),
        },

        _ => return Ok(false),
    }

    tracing::trace!("[NewEvent] {:#?}", &event);

    match state.lock().await.workspaces.get_mut(&root) {
        Some(ws) => {
            for (_, nvim) in ws.clients.iter() {
                if let Err(e) = nvim.exec(COMPILE_START_MSG.into(), false).await {
                    tracing::error!("Fail to echo message to nvim clients {e}")
                }
            }

            if let Err(e) = ws.on_directory_change(path, &event.kind).await {
                tracing::error!("{:?}:\n {:#?}", event, e);

                for (_, nvim) in ws.clients.iter() {
                    if let Err(e) = nvim.log_error("CompileCommands", &e).await {
                        tracing::error!("Fail to echo error to nvim clients {e}")
                    }
                }
            } else {
                tracing::info!("Regenerated compile commands");
                for (_, nvim) in ws.clients.iter() {
                    if let Err(e) = nvim.exec(COMPILE_SUCC_MESSAGE.into(), false).await {
                        tracing::error!("Fail to echo message to nvim clients {e}")
                    }
                }
            }

            let mut debounce = debounce.lock().await;
            *debounce = std::time::SystemTime::now();
        }

        // NOTE: should stop watch here?
        None => return Ok(false),
    };
    Ok(true)
}
