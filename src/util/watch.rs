//! Function to watch file system
//!
//! Mainly used for creation/removal of files and editing of xcodegen config.
use notify::{Error, Event, RecommendedWatcher, RecursiveMode, Watcher};
#[cfg(feature = "daemon")]
use std::{path::Path, time::Duration};
#[cfg(feature = "async")]
use tokio::sync::{mpsc, Mutex};
use wax::{Glob, Pattern};

/// Create new handler to watch workspace root.
#[cfg(feature = "daemon")]
pub fn handler(
    state: crate::daemon::DaemonState,
    root: String,
) -> tokio::task::JoinHandle<anyhow::Result<()>> {
    // NOTE: should watch for registered directories?
    // TODO: Support provideing additional ignore wildcard
    //
    // Some files can be generated as direct result of running build command.
    // In my case this `Info.plist`.
    //
    // For example,  define key inside project.yml under xcodebase key, ignoreGlob of type array.

    let mut debounce = Box::new(std::time::SystemTime::now());
    tokio::spawn(async move {
        let (tx, mut rx) = mpsc::channel(100);

        let mut watcher = RecommendedWatcher::new(move |res: Result<Event, Error>| {
            if let Ok(event) = res {
                tx.blocking_send(event).unwrap();
            } else {
                tracing::error!("Watch Error: {:?}", res);
            };
        })?;

        watcher.watch(Path::new(&root), RecursiveMode::Recursive)?;
        watcher.configure(notify::Config::NoticeEvents(true))?;

        // HACK: ignore seen paths.
        let last_seen = std::sync::Arc::new(Mutex::new(String::default()));

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

            let last_run = debounce.elapsed().unwrap().as_millis();
            let pass_threshold = last_run > 1;
            if !pass_threshold {
                #[cfg(feature = "logging")]
                tracing::debug!("{:?}, paths: {:?}", event.kind, &event.paths);
                tracing::trace!(
                    "   milliseconds: {} pass_threshold: {pass_threshold}, {:?}",
                    debounce.elapsed().unwrap().as_millis(),
                    event
                );
                continue;
            }

            // NOTE: maybe better handle in tokio::spawn?
            match &event.kind {
                notify::EventKind::Create(_) => {
                    tokio::time::sleep(Duration::new(1, 0)).await;
                    #[cfg(feature = "logging")]
                    tracing::debug!("[FileCreated]: {:?}", path);
                }
                notify::EventKind::Remove(_) => {
                    tokio::time::sleep(Duration::new(1, 0)).await;
                    #[cfg(feature = "logging")]
                    tracing::debug!("[FileRemoved]: {:?}", path);
                }
                notify::EventKind::Modify(m) => match m {
                    notify::event::ModifyKind::Data(e) => match e {
                        notify::event::DataChange::Content => {
                            if !path_string.contains("project.yml") {
                                continue;
                            }
                            tokio::time::sleep(Duration::new(1, 0)).await;
                            #[cfg(feature = "logging")]
                            tracing::debug!("[XcodeGenConfigUpdate]");
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
                        #[cfg(feature = "logging")]
                        tracing::debug!("[FileRenamed]: {:?}", path);
                    }
                    _ => continue,
                },

                _ => continue,
            }

            #[cfg(feature = "logging")]
            tracing::trace!("[NewEvent] {:#?}", &event);

            // let mut state = state.lock().await;

            match state.lock().await.workspaces.get_mut(&root) {
                Some(w) => {
                    if let Err(e) = w.on_directory_change(path, &event.kind).await {
                        #[cfg(feature = "logging")]
                        tracing::error!("Fail to process {:?}:\n {:#?}", event, e);
                    }
                    debounce = Box::new(std::time::SystemTime::now())
                }

                // NOTE: should stop watch here
                None => continue,
            };
        }
        Ok(())
    })
}

/// HACK: ignore seen paths.
///
/// Sometiems we get event for the same path, particularly
/// `ModifyKind::Name::Any` is ommited twice for the new path
/// and once for the old path.
///
/// This will compare last_seen with path, updates `last_seen` if not match,
/// else returns true.
#[cfg(feature = "async")]
async fn should_ignore(last_seen: std::sync::Arc<Mutex<String>>, path: &str) -> bool {
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
async fn get_ignore_patterns(state: crate::daemon::DaemonState, root: &String) -> Vec<String> {
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

    // FIXME: Adding extra ignore patterns to `ignore` local config requires restarting daemon.
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
