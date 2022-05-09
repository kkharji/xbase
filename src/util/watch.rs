//! Function to watch file system
//!
//! Mainly used for creation/removal of files and editing of xcodegen config.
use crate::daemon::state::Project;
use crate::daemon::DaemonState;
use crate::{compile, xcodegen};
use notify::event::ModifyKind;
use notify::{Error, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::{future::Future, path::PathBuf, sync::Arc, time::SystemTime};
use std::{path::Path, time::Duration};
use tokio::sync::{mpsc, Mutex};
use tokio::task::JoinHandle;
use wax::{Glob, Pattern};

const COMPILE_START_MSG: &str = "echo 'xcodebase: ⚙ Regenerating compilation database ..'";
const COMPILE_SUCC_MESSAGE: &str = "echo 'xcodebase: ✅ Compilation database regenerated.'";

pub enum WatchError {
    Stop(String),
    Continue(String),
}

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
        if crate::xcodegen::is_valid(ws) {
            patterns.extend(ws.project.config().ignore.clone());
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
    Fut: Future<Output = std::result::Result<bool, WatchError>> + Send,
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
                match e {
                    WatchError::Stop(e) => {
                        tracing::error!("aborting watch service: {e} ... ");
                        break;
                    }
                    WatchError::Continue(e) => {
                        tracing::error!("{e}");
                        continue;
                    }
                }
            }
        }
        Ok(())
    })
}

// TOOD(watch): Fix build/run and compile failure conflict on rename
//
// When watch build exists, if a rename happens, the watcher fails
pub async fn recompile_handler(
    state: DaemonState,
    root: String,
    path: PathBuf,
    event: Event,
    last_seen: Arc<Mutex<String>>,
    debounce: Arc<Mutex<SystemTime>>,
) -> Result<bool, WatchError> {
    match &event.kind {
        EventKind::Create(_) => {
            tokio::time::sleep(Duration::new(1, 0)).await;
            tracing::debug!("[FileCreated]: {:?}", path);
        }
        EventKind::Remove(_) => {
            tokio::time::sleep(Duration::new(1, 0)).await;
            tracing::debug!("[FileRemoved]: {:?}", path);
        }
        EventKind::Modify(m) => match m {
            ModifyKind::Data(e) => match e {
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
            ModifyKind::Name(_) => {
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

    let mut state = state.lock().await;
    let ws = state
        .get_mut_workspace(&root)
        .map_err(|e| WatchError::Stop(e.to_string()))?;

    ws.message_all_nvim_instances(COMPILE_START_MSG).await;

    let name = ws.name();

    if xcodegen::regenerate(name, path, &ws.root)
        .await
        .map_err(|e| WatchError::Stop(e.to_string()))?
    {
        tracing::info!("Updating State.{name}.Project");
        ws.project = match Project::new(&ws.root).await {
            Ok(p) => p,
            Err(e) => {
                let msg = format!("Fail to update project {e}");
                ws.message_all_nvim_instances(&msg).await;
                tracing::error!("{}", msg);
                return Err(WatchError::Continue(msg));
            }
        };

        if let Err(e) = ws.update_lua_state().await {
            ws.message_all_nvim_instances(&e.to_string()).await;
        }
    };

    if compile::ensure_server_config(&ws.root).await.is_err() {
        ws.message_all_nvim_instances("Fail to ensure build server configuration!")
            .await
    };

    if compile::update_compilation_file(&ws.root).await.is_err() {
        ws.message_all_nvim_instances("Fail to regenerate compilation database!")
            .await
    };

    tracing::info!("Regenerated compile commands");
    ws.message_all_nvim_instances(COMPILE_SUCC_MESSAGE).await;

    let mut debounce = debounce.lock().await;
    *debounce = std::time::SystemTime::now();
    Ok(true)
}
