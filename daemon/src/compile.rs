//! Module for generating Compilation Database.
use crate::watch::Event;
use crate::{state::State, Result};
use std::path::PathBuf;
use tap::Pipe;
use tokio::{io::AsyncWriteExt, sync::MutexGuard};
use xbase_proto::Client;

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
    let Client { root, pid, .. } = client;
    let ref name = client.abbrev_root();

    let compile_path = root.join(".compile");
    let compile_exists = compile_path.exists();
    let is_swift_project = root.join("Package.swift").exists();

    if !is_swift_project && ensure_server_config(root).await.is_err() {
        "fail to ensure build server configuration!"
            .pipe(|msg| state.clients.echo_err(root, name, msg))
            .await;
    }

    if let Some(event) = event {
        let project = state.projects.get_mut(root)?;
        let name = project.name().to_string();
        if project.should_generate(event) {
            if let Err(e) = project.generate().await {
                let mut lines = e
                    .to_string()
                    .split("\n")
                    .map(ToString::to_string)
                    .collect::<Vec<String>>();
                if lines.len() == 1 {
                    state.clients.get(&pid)?.echo_err(&lines[0]).await?;
                } else {
                    let first = lines.remove(0);
                    state.clients.get(&pid)?.echo_err(&first).await?;
                    let mut logger = state.clients.get(&pid)?.logger();
                    logger.set_status_end(false, true).await?;
                    logger.set_title(name);
                    logger.append(first).await?;
                    logger.append(lines.join("\n")).await?;
                }
                return Ok(false);
            };

            project.update_compile_database().await?;
            state
                .clients
                .get(&pid)?
                .exec_lua("require'xbase.util'.reload_lsp_servers()", vec![])
                .await?;
            return Ok(true);
        }
    }

    if !is_swift_project && !compile_exists {
        "⚙ Generating compile database (may take few seconds) .."
            .pipe(|msg| state.clients.echo_msg(root, name, msg))
            .await;

        if let Err(err) = state.projects.get(&root)?.update_compile_database().await {
            "setup: fail to regenerate compilation database!"
                .pipe(|msg| state.clients.echo_err(&root, &name, msg))
                .await;

            let mut logger = state.clients.get(&client.pid)?.logger();

            logger.set_running(false).await.ok();

            logger.open_win().await.ok();
            logger.append(err.to_string()).await.ok();

            logger.set_status_end(false, true).await.ok();

            return Ok(false);
        }
        Ok(true)
    } else {
        Ok(false)
    }
}
