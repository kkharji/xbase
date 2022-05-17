use super::{is_seen, WatchArguments, WatchError};
use crate::compile;
use crate::{constants::DAEMON_STATE, types::Client};
use notify::event::{DataChange, ModifyKind};
use notify::{Event, EventKind};
use std::{path::PathBuf, sync::Arc, time::Duration};
use tokio::sync::Mutex;

// TODO(structure): rename or move to compile
const START_MSG: &'static str = "⚙ auto compiling ..";

const SUCC_MESSAGE: &'static str = "✅ compiled";

pub async fn create(args: WatchArguments) -> Result<(), WatchError> {
    let WatchArguments {
        info, path, event, ..
    } = args;

    let info = info.lock_owned().await;
    let Client { root, .. } = info.try_into_project()?;

    if should_skip_compile(&event, &path, args.last_seen).await {
        tracing::trace!("Skipping {:#?}", &event);
        return Ok(());
    }

    tracing::trace!("[NewEvent] {:#?}", &event);

    let state = DAEMON_STATE.clone();
    let mut state = state.lock().await;
    let mut debounce = args.debounce.lock().await;

    let project_name = root.file_name().unwrap().to_string_lossy().to_string();

    echo_messsage_to_clients(&state, root, &project_name, START_MSG).await;

    {
        let res = try_updating_project_state(&mut state, &path, root, &project_name).await;
        if res.is_err() {
            *debounce = std::time::SystemTime::now();
            res?;
        }
    }

    ensure_server_configuration(&state, root, &project_name).await;

    if let Err(e) = generate_compiled_commands(&state, root, &project_name).await {
        *debounce = std::time::SystemTime::now();
        return Err(e);
    }

    echo_messsage_to_clients(&state, root, &project_name, SUCC_MESSAGE).await;

    *debounce = std::time::SystemTime::now();

    Ok(())
}

async fn try_updating_project_state<'a>(
    state: &mut tokio::sync::MutexGuard<'a, crate::state::State>,
    path: &PathBuf,
    root: &PathBuf,
    project_name: &String,
) -> Result<(), WatchError> {
    let generated = match crate::xcodegen::regenerate(path, &root).await {
        Ok(generated) => generated,
        Err(e) => {
            echo_error_to_clients(state, root, project_name, &e.to_string()).await;
            return Ok(());
        }
    };

    if generated {
        state
            .projects
            .get_mut(root)
            .ok_or_else(|| WatchError::Stop(format!("'{:?}' isn't registred project", root)))?
            .update()
            .await
            .map_err(WatchError::r#continue)?;
    }

    Ok(())
}

async fn ensure_server_configuration<'a>(
    state: &'a tokio::sync::MutexGuard<'a, crate::state::State>,
    root: &PathBuf,
    project_name: &String,
) {
    if compile::ensure_server_config(&root).await.is_err() {
        echo_error_to_clients(
            state,
            &root,
            project_name,
            "Fail to ensure build server configuration!",
        )
        .await;
    }
}

async fn generate_compiled_commands<'a>(
    state: &'a tokio::sync::MutexGuard<'a, crate::state::State>,
    root: &PathBuf,
    project_name: &String,
) -> Result<(), WatchError> {
    if let Err(err) = compile::update_compilation_file(&root).await {
        echo_error_to_clients(
            state,
            &root,
            project_name,
            "Fail to regenerate compilation database!",
        )
        .await;

        // use crate::util::proc_exists;
        // for (pid, client) in state.clients.iter() {
        //     if proc_exists(pid, || {}) {
        //         let mut logger = client.new_logger("Compile Error", project_name, &None);
        //         logger.set_running().await.ok();
        //         let ref win = Some(logger.open_win().await.map_err(WatchError::r#continue)?);
        //         logger.log(err.to_string(), win).await.ok();
        //         logger.set_status_end(false, true).await.ok();
        //     }
        // }
        return Err(WatchError::r#continue(err));
    }

    Ok(())
}

async fn echo_messsage_to_clients<'a>(
    state: &tokio::sync::MutexGuard<'a, crate::state::State>,
    root: &PathBuf,
    project_name: &String,
    msg: &str,
) {
    state.clients.echo_msg(&root, project_name, msg).await;
}

async fn echo_error_to_clients<'a>(
    state: &tokio::sync::MutexGuard<'a, crate::state::State>,
    root: &PathBuf,
    project_name: &String,
    msg: &str,
) {
    state.clients.echo_err(&root, project_name, msg).await;
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
