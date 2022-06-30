//! Module for generating Compilation Database.
use crate::broadcast::{self, Broadcast};
use crate::watch::Event;
use crate::{constants::State, Result};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::{io::AsyncWriteExt, sync::MutexGuard};

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
    root: &PathBuf,
    event: Option<&Event>,
    broadcast: &Arc<Broadcast>,
) -> Result<bool> {
    let compile_path = root.join(".compile");
    let compile_exists = compile_path.exists();
    let is_swift_project = root.join("Package.swift").exists();

    if !is_swift_project && ensure_server_config(root).await.is_err() {
        broadcast::notify_error!(broadcast, "fail to ensure build server configuration!")?;
    };

    if let Some(event) = event {
        let project = state.projects.get_mut(root)?;
        if project.should_generate(event) {
            if let Err(e) = project.generate(broadcast).await {
                for line in e.to_string().split("\n") {
                    broadcast::notify_error!(broadcast, "{line}")?;
                }
                return Ok(false);
            };
            project.update_compile_database(broadcast).await?;
            return Ok(true);
        }
    }

    if !is_swift_project && !compile_exists {
        broadcast::notify_info!(
            broadcast,
            "⚙ Generating compile database (may take few seconds) .."
        )?;

        if let Err(err) = state
            .projects
            .get(&root)?
            .update_compile_database(broadcast)
            .await
        {
            broadcast::notify_error!(
                broadcast,
                "Fail to regenerate compilation database!, checkout logs"
            )?;
            for line in err.to_string().split("\n") {
                broadcast::log_error!(broadcast, "{line}")?;
            }

            return Ok(false);
        }
        Ok(true)
    } else {
        Ok(false)
    }
}
