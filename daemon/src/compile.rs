//! Module for generating Compilation Database.
use crate::logger::Logger;
use crate::watch::Event;
use crate::{constants::State, Result};
use std::path::PathBuf;
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
    logger: &Logger,
) -> Result<bool> {
    let Client { root, pid, .. } = client;
    let ref name = client.abbrev_root();

    let compile_path = root.join(".compile");
    let compile_exists = compile_path.exists();
    let is_swift_project = root.join("Package.swift").exists();

    if !is_swift_project && ensure_server_config(root).await.is_err() {
        logger.error("fail to ensure build server configuration!");
    };

    if let Some(event) = event {
        let project = state.projects.get_mut(root)?;
        let name = project.name().to_string();
        if project.should_generate(event) {
            if let Err(e) = project.generate().await {
                e.to_string().split("\n").for_each(|l| {
                    logger.error(l);
                });
                return Ok(false);
            };

            // TODO: pass logger
            project.update_compile_database().await?;
            return Ok(true);
        }
    }

    if !is_swift_project && !compile_exists {
        logger.append("⚙ Generating compile database (may take few seconds) ..");

        if let Err(err) = state.projects.get(&root)?.update_compile_database().await {
            logger.error("setup: fail to regenerate compilation database!");
            err.to_string().split("\n").for_each(|l| {
                logger.append(l);
            });

            return Ok(false);
        }
        Ok(true)
    } else {
        Ok(false)
    }
}
