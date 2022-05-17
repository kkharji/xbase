use crate::error::WatchError;
use crate::{daemon::WatchTarget, types::Client};
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;
use tap::Pipe;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

mod project;
mod target;

#[derive(Debug)]
pub enum Info {
    Project(Client),
    Target(WatchTarget),
}

impl Info {
    pub fn try_into_target(&self) -> Result<&WatchTarget, WatchError> {
        if let Self::Target(v) = self {
            Ok(v)
        } else {
            Err(WatchError::Stop(
                "Expected WatchStart got something else".into(),
            ))
        }
    }

    pub fn try_into_project(&self) -> Result<&Client, WatchError> {
        if let Self::Project(v) = self {
            Ok(v)
        } else {
            Err(WatchError::Stop(
                "Expected Client got something else".into(),
            ))
        }
    }
}

pub struct WatchArguments {
    info: Arc<Mutex<Info>>,
    path: PathBuf,
    event: notify::Event,
    last_seen: Arc<Mutex<String>>,
    debounce: Arc<Mutex<SystemTime>>,
}

#[derive(Debug)]
pub struct WatchHandler(tokio::task::JoinHandle<Result<()>>);

impl WatchHandler {
    pub fn new_project_watcher(client: Client, ignore_pattern: Vec<String>) -> Self {
        new_handler(Info::Project(client), ignore_pattern, project::create)
    }

    pub fn new_target_watcher(req: WatchTarget, ignore_pattern: Vec<String>) -> Self {
        new_handler(Info::Target(req), ignore_pattern, target::create)
    }

    pub fn inner(&self) -> &tokio::task::JoinHandle<Result<()>> {
        &self.0
    }
}

/// Create auto compile project handler
fn new_handler<'a, F, Fut>(
    info: Info,
    ignore_pattern: Vec<String>,
    event_handler: F,
) -> WatchHandler
where
    F: Fn(WatchArguments) -> Fut + Send + 'static + Copy,
    Fut: std::future::Future<Output = std::result::Result<(), WatchError>> + Send,
{
    use tokio::sync::mpsc::channel;

    let debounce = Arc::new(Mutex::new(SystemTime::now()));
    let last_seen = Arc::new(Mutex::new(String::default()));

    let (tx, mut rx) = channel::<notify::Event>(1);

    let (root, target) = match &info {
        Info::Project(client) => (client.root.clone(), None),
        Info::Target(req) => (
            req.client.root.clone(),
            req.config.target.clone().pipe(Some),
        ),
    };
    let info = Arc::new(Mutex::new(info));

    WatchHandler(tokio::spawn(async move {
        let _watcher = root_watcher(root.as_path(), tx)?;

        if let Some(target) = target {
            tracing::info!("WatchTarget({target})");
        } else {
            tracing::info!("Watch({})", root.display());
        }

        let ignore_patterns = ignore_pattern
            .iter()
            .map(AsRef::as_ref)
            .collect::<Vec<&str>>();
        let ignore = match wax::any::<wax::Glob, _>(ignore_patterns) {
            Ok(i) => i,
            Err(err) => {
                anyhow::bail!("Fail to generate ignore glob: {err}")
            }
        };

        while let Some(event) = rx.recv().await {
            let (last_seen, debounce) = (last_seen.clone(), debounce.clone());
            let (path, skip) = should_skip_event(&ignore, &event, debounce.clone()).await;

            if skip {
                continue;
            }

            let response = WatchArguments {
                info: info.clone(),
                path: path.unwrap(),
                event,
                last_seen,
                debounce,
            }
            .pipe(event_handler)
            .await;

            if let Err(e) = response {
                match e {
                    WatchError::Stop(e) => {
                        tracing::error!("aborting watch service: {e} ... ");
                        break;
                    }

                    WatchError::Continue(e) => {
                        tracing::error!("{e}");
                        continue;
                    }
                    _ => break,
                }
            }
        }

        anyhow::Ok(())
    }))
}

impl serde::Serialize for WatchHandler {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bool(true)
    }
}

impl<'de> serde::Deserialize<'de> for WatchHandler {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct UnitVisitor;

        impl<'de> serde::de::Visitor<'de> for UnitVisitor {
            type Value = WatchHandler;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("unit")
            }

            fn visit_unit<E>(self) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(WatchHandler(tokio::spawn(async { anyhow::Ok(()) })))
            }
        }

        deserializer.deserialize_unit(UnitVisitor)
    }
}

fn root_watcher(root: &Path, tx: Sender<notify::Event>) -> Result<notify::FsEventWatcher> {
    use notify::{Config::*, RecommendedWatcher, RecursiveMode::Recursive, Watcher};

    let mut watcher = <RecommendedWatcher as Watcher>::new(move |res| {
        if let Ok(event) = res {
            if let Err(err) = tx.blocking_send(event) {
                tracing::error!("Fail send event {err}");
            };
        } else {
            tracing::error!("Watch Error: {:?}", res);
        };
    })?;

    Watcher::watch(&mut watcher, root, Recursive)?;
    Watcher::configure(&mut watcher, NoticeEvents(true))?;

    Ok(watcher)
}

/// HACK: ignore seen paths.
///
/// Sometimes we get event for the same path, particularly `ModifyKind::Name::Any` is omitted twice
/// This will compare last_seen with path, updates `last_seen` if not match, else returns true.
async fn is_seen(last_seen: Arc<Mutex<String>>, path: &str) -> bool {
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
    ignore: &wax::Any<'a>,
    event: &notify::Event,
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

    if wax::Pattern::is_match(ignore, &*path_string) {
        return (Some(path), true);
    }

    let last_run = match debounce.lock().await.elapsed() {
        Ok(time) => time.as_millis(),
        Err(err) => {
            tracing::error!("Fail to get last_run time: {err}");
            return (Some(path), true);
        }
    };

    if !(last_run > 1) {
        tracing::debug!("{:?}, paths: {:?}", event.kind, &event.paths);
        tracing::trace!("pass_threshold: {last_run}, {:?}", event);
        return (Some(path), true);
    }

    return (Some(path), false);
}
