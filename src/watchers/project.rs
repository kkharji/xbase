use super::*;
use crate::{compile, xcodegen};
use crate::{daemon::Workspace, state::DaemonState, types::Project};
use anyhow::Result;
use notify::{event::ModifyKind, Event, EventKind};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tap::Pipe;
use tokio::{sync::Mutex, task::JoinHandle, time::sleep};
use tracing::*;

const COMPILE_START_MSG: &str = "echo 'xcodebase: ⚙ Regenerating compilation database ..'";
const COMPILE_SUCC_MESSAGE: &str = "echo 'xcodebase: ✅ Compilation database regenerated.'";

pub type ProjectWatchersInner = HashMap<String, JoinHandle<Result<()>>>;

#[derive(Default, Debug)]
pub struct ProjectWatchers(ProjectWatchersInner);

impl Deref for ProjectWatchers {
    type Target = ProjectWatchersInner;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ProjectWatchers {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// TOOD(watch): Fix build/run and compile failure conflict on rename
//
// When watch build exists, if a rename happens, the watcher fails
pub async fn recompile_handler(req: WatchArguments) -> Result<bool, WatchError> {
    let WatchArguments {
        state,
        root,
        path,
        event,
        last_seen,
        debounce,
        key,
    } = req;

    if should_skip(&event, &path, last_seen).await {
        return Ok(false);
    }

    trace!("[NewEvent] {:#?}", &event);

    let mut state = state.lock().await;
    let ws = state
        .get_mut_workspace(&root)
        .map_err(|e| WatchError::Stop(e.to_string()))?;

    ws.clients.log_info(COMPILE_START_MSG).await;

    let name = ws.name();
    let generated = xcodegen::regenerate(name, path, &ws.root)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            ws.clients.log_error(&msg);
            WatchError::Stop(msg)
        })?;

    if generated {
        ws.project = match create_new_project(ws).await {
            Ok(value) => value,
            Err(value) => return value,
        };

        if let Err(e) = ws.sync_state().await {
            ws.clients.log_error(&e.to_string()).await;
        }
    };

    if compile::ensure_server_config(&ws.root).await.is_err() {
        ws.clients
            .log_error("Fail to ensure build server configuration!")
            .await
    };

    if compile::update_compilation_file(&ws.root).await.is_err() {
        ws.clients
            .log_error("Fail to regenerate compilation database!")
            .await
    };

    ws.clients.log_info(COMPILE_SUCC_MESSAGE).await;

    let mut debounce = debounce.lock().await;
    *debounce = std::time::SystemTime::now();

    Ok(true)
}

async fn create_new_project(ws: &mut Workspace) -> Result<Project, Result<bool, WatchError>> {
    Ok(match Project::new(&ws.root).await {
        Ok(p) => {
            let msg = format!("Updating State.{}.Project", ws.name());
            ws.clients.log_info(&msg).await;
            p
        }
        Err(e) => {
            let msg = format!("Fail to update project {e}");
            ws.clients.log_error(&msg).await;
            return Err(Err(WatchError::Continue(msg)));
        }
    })
}

async fn should_skip(event: &Event, path: &PathBuf, last_seen: Arc<Mutex<String>>) -> bool {
    match &event.kind {
        EventKind::Create(_) => {
            sleep(Duration::new(1, 0)).await;
            debug!("[FileCreated]: {:?}", path);
        }
        EventKind::Remove(_) => {
            sleep(Duration::new(1, 0)).await;
            debug!("[FileRemoved]: {:?}", path);
        }
        EventKind::Modify(m) => match m {
            ModifyKind::Data(e) => match e {
                notify::event::DataChange::Content => {
                    if !path.display().to_string().contains("project.yml") {
                        return true;
                    }
                    sleep(Duration::new(1, 0)).await;
                    debug!("[XcodeGenConfigUpdate]");
                    // HACK: Not sure why, but this is needed because xcodegen break.
                }
                _ => return true,
            },
            ModifyKind::Name(_) => {
                let path_string = path.to_string_lossy();
                // HACK: only account for new path and skip duplications
                if !path.exists() || should_ignore(last_seen.clone(), &path_string).await {
                    return true;
                }
                sleep(Duration::new(1, 0)).await;
                debug!("[FileRenamed]: {:?}", path);
            }
            _ => return true,
        },

        _ => return true,
    };

    false
}
