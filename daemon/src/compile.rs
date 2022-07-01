//! Module for generating Compilation Database.
use crate::broadcast::Broadcast;
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
) -> Result<()> {
    let compile_path = root.join(".compile");
    let compile_exists = compile_path.exists();
    let is_swift_project = root.join("Package.swift").exists();

    if !is_swift_project && ensure_server_config(root).await.is_err() {
        broadcast.error("fail to ensure build server configuration ");
    };

    if let Some(event) = event {
        let project = state.projects.get_mut(root)?;
        if project.should_generate(event) {
            project.generate(broadcast).await?;
            project.update_compile_database(broadcast).await?;
        }
    }

    if !is_swift_project && !compile_exists {
        state
            .projects
            .get(&root)?
            .update_compile_database(broadcast)
            .await
            .ok();
    }
    Ok(())
}
