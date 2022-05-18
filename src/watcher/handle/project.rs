use super::{is_seen, WatchArguments, WatchError};
use crate::compile;
use crate::{constants::DAEMON_STATE, types::Client};
use notify::event::{DataChange, ModifyKind};
use notify::{Event, EventKind};
use std::{path::PathBuf, sync::Arc, time::Duration};
use tokio::sync::Mutex;

// TODO(structure): rename or move to compile
const START_MSG: &'static str = "⚙ recompiling project ..";

const SUCC_MESSAGE: &'static str = "✅ compiled";

pub async fn create(args: WatchArguments) -> Result<(), WatchError> {
    let WatchArguments {
        info, path, event, ..
    } = args;

    let info = info.lock_owned().await;
    let Client { root, .. } = info.try_into_project()?;

    if should_skip_compile(&event, &path, args.last_seen).await {
        tracing::debug!("Skipping {:?}", &event.paths);
        return Ok(());
    }

    tracing::trace!("[NewEvent] {:#?}", &event);

    let ref name = root.file_name().unwrap().to_string_lossy().to_string();
    let ref mut state = DAEMON_STATE.clone().lock_owned().await;
    let mut debounce = args.debounce.lock().await;

    state.clients.echo_msg(&root, name, START_MSG).await;

    if let Err(e) = compile::ensure_server_support(state, name, root, Some(&path)).await {
        *debounce = std::time::SystemTime::now();
        return Err(e.into());
    }

    state.clients.echo_msg(&root, name, SUCC_MESSAGE).await;

    *debounce = std::time::SystemTime::now();

    Ok(())
}

async fn should_skip_compile(event: &Event, path: &PathBuf, last_seen: Arc<Mutex<String>>) -> bool {
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
                DataChange::Content => {
                    if !path.display().to_string().contains("project.yml") {
                        return true;
                    }
                    tokio::time::sleep(Duration::new(1, 0)).await;
                    tracing::debug!("[XcodeGenConfigUpdate]");
                    // HACK: Not sure why, but this is needed because xcodegen break.
                }
                _ => return true,
            },

            ModifyKind::Name(_) => {
                let path_string = path.to_string_lossy();
                // HACK: only account for new path and skip duplications
                if !path.exists() || is_seen(last_seen.clone(), &path_string).await {
                    return true;
                }
                tokio::time::sleep(Duration::new(1, 0)).await;
                tracing::debug!("[FileRenamed]: {:?}", path);
            }
            _ => return true,
        },

        _ => return true,
    };

    false
}
