use super::{WatchArguments, WatchError};
use crate::daemon::WatchStart;
use crate::nvim::BulkLogRequest;
use crate::types::WatchType;
use crate::types::XTarget;
use crate::xcode;
use anyhow::Result;
use notify::event::ModifyKind;
use notify::{Event, EventKind};
use std::collections::HashMap;
use std::ops::Deref;
use std::ops::DerefMut;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tracing::debug;

pub type TargetWatchersInner = HashMap<XTarget, (WatchStart, JoinHandle<Result<()>>)>;

#[derive(Debug, Default)]
pub struct TargetWatchers {
    root: String,
    watchers: TargetWatchersInner,
}

impl Deref for TargetWatchers {
    type Target = TargetWatchersInner;
    fn deref(&self) -> &Self::Target {
        &self.watchers
    }
}

impl DerefMut for TargetWatchers {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.watchers
    }
}

pub async fn event_handler(req: WatchArguments) -> Result<bool, WatchError> {
    if should_ignore(&req.event, req.path, req.last_seen).await {
        return Ok(false);
    }

    debug!("Rebuilding for {:#?}", &req.event);

    let mut state = req.state.lock().await;
    let (watch_req, handler) = match state.target_watchers.get(&req.key) {
        Some(value) => value,
        None => return Err(WatchError::Stop("No Handler found".into())),
    };

    let (client, request) = (watch_req.client.clone(), &watch_req.request);

    let workspace = state
        .get_workspace(&req.root)
        .map_err(|e| WatchError::Stop(e.to_string()))?;
    let nvim = workspace
        .nvim(&client.pid)
        .map_err(|e| WatchError::Stop(e.to_string()))?;

    let stream = match request.watch_type {
        WatchType::Build => xcode::stream(&req.root, &["build"], watch_req.request.clone())
            .await
            .map_err(|e| WatchError::Continue(format!("Build Failed: {e}")))?,
        WatchType::Run => {
            nvim.log_error("Watch", "Run is not supported yet! .. aborting")
                .await
                .map_err(|e| WatchError::Stop(format!("Unable to log to nvim buffer: {e}")))?;

            state.validate(Some(client)).await;

            return Err(WatchError::Stop("Run not supported yet!".into()));
        }
    };

    nvim.buffers
        .log
        .bulk_append(BulkLogRequest {
            nvim,
            title: "Watch",
            direction: None,
            stream,
            clear: false,
            open: false,
        })
        .await
        .map_err(|e| WatchError::Continue(format!("Logging to client failed: {e}")))?;

    let mut debounce = req.debounce.lock().await;
    *debounce = SystemTime::now();

    Ok(true)
}

async fn should_ignore(event: &Event, path: PathBuf, last_seen: Arc<Mutex<String>>) -> bool {
    async fn should_ignore(event: &Event, path: PathBuf, last_seen: Arc<Mutex<String>>) -> bool {
        if let EventKind::Modify(ModifyKind::Name(_)) = &event.kind {
            let path_string = path.to_string_lossy();
            // HACK: only account for new path and skip duplications
            if !path.exists() || super::should_ignore(last_seen.clone(), &path_string).await {
                return true;
            }
            sleep(Duration::new(1, 0)).await;
        }
        false
    }

    fn is_rename_event(event: &Event) -> bool {
        matches!(
            event.kind,
            notify::EventKind::Modify(notify::event::ModifyKind::Name(_))
        )
    }

    fn is_create_event(event: &Event) -> bool {
        matches!(event.kind, notify::EventKind::Create(_))
    }

    fn is_modified_event(event: &Event) -> bool {
        matches!(
            event.kind,
            notify::EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Content
            ))
        )
    }

    !(is_modified_event(event) || is_create_event(event) || is_rename_event(event))
        || should_ignore(event, path, last_seen).await
}
