mod project;
mod targets;
pub use project::*;
pub use targets::*;

use crate::state::DaemonState;
use anyhow::Result;
use notify::{Event, FsEventWatcher, RecommendedWatcher, RecursiveMode, Watcher};
use std::{future::Future, path::Path, path::PathBuf, sync::Arc, time::SystemTime};
use tap::Pipe;

use tokio::sync::{self, mpsc::Sender, Mutex};
use tokio::task::JoinHandle;

use tracing::*;
use wax::{Any, Glob, Pattern};

pub enum WatchError {
    Stop(String),
    Continue(String),
}

pub struct WatchArguments {
    state: DaemonState,
    root: String,
    path: PathBuf,
    event: Event,
    last_seen: Arc<Mutex<String>>,
    debounce: Arc<Mutex<SystemTime>>,
    key: String,
}

/// Create new watcher handler using notify.
/// NOTE: should watch for registered directories only?
fn new_watcher(root: &str, tx: Sender<Event>) -> Result<FsEventWatcher> {
    let mut watcher = RecommendedWatcher::new(move |res: Result<Event, notify::Error>| {
        if let Ok(event) = res {
            if let Err(err) = tx.blocking_send(event) {
                error!("Fail send event {err}");
            };
        } else {
            error!("Watch Error: {:?}", res);
        };
    })?;

    watcher.watch(Path::new(root), RecursiveMode::Recursive)?;
    watcher.configure(notify::Config::NoticeEvents(true))?;

    Ok(watcher)
}

pub fn new_watch_handler<F, Fut>(
    root: String,
    state: DaemonState,
    event_handler: F,
    key: Option<String>,
) -> JoinHandle<Result<()>>
where
    F: Fn(WatchArguments) -> Fut + Send + 'static,
    Fut: Future<Output = std::result::Result<bool, WatchError>> + Send,
{
    let debounce = Arc::new(Mutex::new(SystemTime::now()));

    tokio::spawn(async move {
        let (tx, mut rx) = sync::mpsc::channel(1);
        let watcher = new_watcher(&root, tx);
        let last_seen = Arc::new(Mutex::new(String::default()));
        let patterns = state
            .clone()
            .lock()
            .await
            .pipe(|s| s.get_workspace(&root).map(|w| w.ignore_patterns.clone()))?;

        let patterns = patterns.iter().map(AsRef::as_ref).collect::<Vec<&str>>();
        let ignore = match wax::any::<Glob, _>(patterns) {
            Ok(i) => i,
            Err(err) => {
                anyhow::bail!("Fail to generate ignore glob: {err}")
            }
        };

        while let Some(event) = rx.recv().await {
            let (path, skip) = should_skip_event(&ignore, &event, debounce.clone()).await;

            if skip {
                continue;
            }

            let future = event_handler(WatchArguments {
                state: state.clone(),
                root: root.clone(),
                path: path.unwrap(),
                event,
                last_seen: last_seen.clone(),
                debounce: debounce.clone(),
                key: key.clone().unwrap_or_default(),
            });

            if let Err(e) = future.await {
                match e {
                    WatchError::Stop(e) => {
                        error!("aborting watch service: {e} ... ");
                        break;
                    }
                    WatchError::Continue(e) => {
                        error!("{e}");
                        continue;
                    }
                }
            }
        }

        Ok(())
    })
}

/// HACK: ignore seen paths.
///
/// Sometimes we get event for the same path, particularly `ModifyKind::Name::Any` is omitted twice
/// This will compare last_seen with path, updates `last_seen` if not match, else returns true.
async fn should_ignore(last_seen: Arc<Mutex<String>>, path: &str) -> bool {
    let path = path.to_string();
    // HACK: Always return false for project.yml
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

/// Should skip event
async fn should_skip_event<'a>(
    ignore: &Any<'a>,
    event: &Event,
    debounce: Arc<Mutex<SystemTime>>,
) -> (Option<PathBuf>, bool) {
    let path = match event.paths.get(0) {
        Some(p) => p.clone(),
        None => return (None, true),
    };

    let path_string = match path.to_str() {
        Some(s) => s.to_string(),
        None => return (Some(path), true),
    };

    if ignore.is_match(&*path_string) {
        return (Some(path), true);
    }

    let last_run = match debounce.lock().await.elapsed() {
        Ok(time) => time.as_millis(),
        Err(err) => {
            error!("Fail to get last_run time: {err}");
            return (Some(path), true);
        }
    };

    if !(last_run > 1) {
        debug!("{:?}, paths: {:?}", event.kind, &event.paths);
        trace!("pass_threshold: {last_run}, {:?}", event);
        return (Some(path), true);
    }

    return (Some(path), false);
}
