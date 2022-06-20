//! Module for generating Compilation Database.
use futures::StreamExt;
use xbase_proto::Client;

use crate::watch::Event;
use {
    crate::{state::State, Result},
    std::path::PathBuf,
    tap::Pipe,
    tokio::{fs::metadata, io::AsyncWriteExt, sync::MutexGuard},
};

/// Ensure that buildServer.json exists in root directory.
pub async fn ensure_server_config(root: &PathBuf) -> Result<()> {
    use crate::constants::SERVER_BINARY_PATH;
    use serde_json::json;

    let path = root.join("buildServer.json");
    if tokio::fs::File::open(&path).await.is_ok() {
        return Ok(());
    }

    log::info!("Creating {:?}", path);

    let mut file = tokio::fs::File::create(path).await?;
    file.write_all(
        json!({
            "name": "Xbase",
            "argv": [&*SERVER_BINARY_PATH],
            // TODO(daemon): if buildServer.version ~= 0.2 it should be removed and regenerated
            "version": "0.2",
            "bspVersion": "0.2",
            "languages": ["swift", "objective-c", "objective-cpp", "c", "cpp"]
        })
        .to_string()
        .as_ref(),
    )
    .await?;
    file.sync_all().await?;
    file.shutdown().await?;

    Ok(())
}

pub async fn ensure_server_support<'a>(
    state: &'a mut MutexGuard<'_, State>,
    client: &Client,
    event: Option<&Event>,
) -> Result<bool> {
    let Client { root, .. } = client;
    let ref name = client.abbrev_root();

    let compile_path = root.join(".compile");
    let compile_exists = metadata(compile_path).await.is_ok();

    if ensure_server_config(root).await.is_err() {
        "fail to ensure build server configuration!"
            .pipe(|msg| state.clients.echo_err(root, name, msg))
            .await;
    }

    if event.is_some() {
        match state.projects.get(root)?.regenerate(event).await {
            Ok(Some(mut stream)) => {
                let mut logger = state.clients.get(&client.pid)?.logger();

                let mut success = true;
                logger.set_running(true).await.ok();
                while let Some(output) = stream.next().await {
                    if output.is_exit() {
                        success = output.as_exit().unwrap().eq("0");
                        if !success {
                            logger
                                .append("[ERROR]: Unable to generate xcodeproj")
                                .await?;
                        };
                    } else {
                        logger.append(output).await?;
                    }
                }

                if success {
                    "setup: ⚙ generated xcodeproj ..."
                        .pipe(|msg| state.clients.echo_msg(root, name, msg))
                        .await;
                    logger.close_win().await?;
                }
                state.projects.get_mut(root)?.update(client).await?;
            }
            Ok(None) => {}
            Err(e) => {
                state.clients.echo_err(&root, name, &e.to_string()).await;
                return Err(e);
            }
        };
    } else if compile_exists {
        return Ok(false);
    }

    if event.is_none() {
        "⚙ generating xcodeproj ..."
            .pipe(|msg| state.clients.echo_msg(root, name, msg))
            .await;

        if let Some(err) = match state.projects.get(root)?.regenerate(event).await {
            Ok(Some(mut stream)) => {
                let mut logger = state.clients.get(&client.pid)?.logger();
                let mut success = true;

                logger.set_running(true).await.ok();
                logger.open_win().await?;

                while let Some(output) = stream.next().await {
                    if output.is_exit() {
                        success = output.as_exit().unwrap().eq("0");
                        if !success {
                            logger
                                .append("[ERROR]: Unable to generate xcodeproj")
                                .await?;
                        };
                    } else {
                        logger.append(output).await?;
                    }
                }

                if success {
                    "setup: ⚙ generated xcodeproj ..."
                        .pipe(|msg| state.clients.echo_msg(root, name, msg))
                        .await;
                    logger.close_win().await?;
                }
                state.projects.get_mut(root)?.update(client).await?;
                None
            }
            Ok(None) => None,
            Err(e) => Some(e),
        } {
            let ref msg = format!("fail to generate xcodeproj: {err}");
            state.clients.echo_err(root, name, msg).await;
        }
    };

    if !compile_exists {
        "⚙ Generating compile database .."
            .pipe(|msg| state.clients.echo_msg(root, name, msg))
            .await;
    }

    // The following command won't successed if this file doesn't exists
    if let Err(err) = state.projects.get(&root)?.generate_compile_commands().await {
        "setup: fail to regenerate compilation database!"
            .pipe(|msg| state.clients.echo_err(&root, &name, msg))
            .await;

        let mut logger = state.clients.get(&client.pid)?.logger();

        logger.set_running(false).await.ok();

        logger.open_win().await.ok();
        logger.append(err.to_string()).await.ok();

        logger.set_status_end(false, true).await.ok();

        return Err(err);
    }

    Ok(true)
}
