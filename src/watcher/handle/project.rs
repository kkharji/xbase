use super::{is_seen, WatchArguments, WatchError};
use crate::compile;
use crate::{constants::DAEMON_STATE, types::Client};
use notify::event::{DataChange, ModifyKind};
use notify::{Event, EventKind};
use std::{path::PathBuf, sync::Arc, time::Duration};
use tokio::sync::Mutex;

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

const START_MSG: &'static str = "⚙ auto compiling ..";

const SUCC_MESSAGE: &'static str = "✅ compiled";

pub async fn create(args: WatchArguments) -> Result<(), WatchError> {
    let WatchArguments {
        info,
        path,
        event,
        last_seen,
        debounce,
    } = args;

    let info = info.lock().await;
    let Client { root, .. } = info.try_into_project()?;

    if should_skip_compile(&event, &path, last_seen).await {
        tracing::trace!("Skipping {:#?}", &event);
        return Ok(());
    }

    tracing::trace!("[NewEvent] {:#?}", &event);

    let state = DAEMON_STATE.clone();
    let mut state = state.lock().await;

    let project_name = root.file_name().unwrap().to_string_lossy().to_string();

    state
        .clients
        .echo_msg(&root, &project_name, START_MSG)
        .await;

    let generated = match crate::xcodegen::regenerate(path, &root).await {
        Ok(generated) => generated,
        Err(e) => {
            state
                .clients
                .echo_err(&root, &project_name, &e.to_string())
                .await;

            return Ok(());
        }
    };

    if generated {
        if let Some(project) = state.projects.get_mut(root) {
            tracing::info!("Updating {} project internal state", project_name);
            project.update().await.map_err(WatchError::r#continue)?;
        }
    }

    if compile::ensure_server_config(&root).await.is_err() {
        state
            .clients
            .echo_err(
                &root,
                &project_name,
                "Fail to ensure build server configuration!",
            )
            .await;
    };

    if compile::update_compilation_file(&root).await.is_err() {
        state
            .clients
            .echo_err(
                &root,
                &project_name,
                "Fail to regenerate compilation database!",
            )
            .await;
    };
    tracing::info!("updated {} .compile", project_name);

    state
        .clients
        .echo_msg(&root, &project_name, SUCC_MESSAGE)
        .await;

    let mut debounce = debounce.lock().await;
    *debounce = std::time::SystemTime::now();

    Ok(())
}
