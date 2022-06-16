//! Module for generating Compilation Database.
//!
//! based on <https://clang.llvm.org/docs/JSONCompilationDatabase.html>
//!
//! see <https://github.com/apple/sourcekit-lsp/blob/main/Sources/SKCore/CompilationDatabase.swift>

use {
    crate::{client::Client, error::XcodeGenError, state::State, util::pid, xcodegen, Result},
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

    tracing::info!("Creating {:?}", path);

    let mut file = tokio::fs::File::create(path).await?;
    file.write_all(
        json!({
            "name": "Xbase",
            "argv": [SERVER_BINARY_PATH],
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
    path: Option<&PathBuf>,
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

    if let Some(path) = path {
        let generated = match xcodegen::regenerate(path, &root).await {
            Ok(generated) => generated,
            Err(e) => {
                state.clients.echo_err(&root, name, &e.to_string()).await;
                return Err(e);
            }
        };

        if generated {
            state.projects.get_mut(root)?.update().await?
        }
    } else if compile_exists {
        return Ok(false);
    }

    if xcodegen::is_valid(&root) && path.is_none() {
        "⚙ generating xcodeproj ..."
            .pipe(|msg| state.clients.echo_msg(root, name, msg))
            .await;

        if let Some(err) = match xcodegen::generate(&root).await {
            Ok(status) => {
                if status.success() {
                    "setup: ⚙ generate xcodeproj ..."
                        .pipe(|msg| state.clients.echo_msg(root, name, msg))
                        .await;
                    None
                } else {
                    Some(XcodeGenError::XcodeProjUpdate(name.into()).into())
                }
            }
            Err(e) => Some(e),
        } {
            let ref msg = format!("fail to generate xcodeproj: {err}");
            state.clients.echo_err(root, name, msg).await;
        }
    };

    // TODO(compile): check for .xcodeproj if project.yml is not generated
    if !compile_exists {
        "⚙ generating compile database .."
            .pipe(|msg| state.clients.echo_msg(root, name, msg))
            .await;
    }

    // The following command won't successed if this file doesn't exists
    if let Err(err) = state.projects.get(&root)?.generate_compile_commands().await {
        "setup: fail to regenerate compilation database!"
            .pipe(|msg| state.clients.echo_err(&root, &name, msg))
            .await;

        for (pid, nvim) in state.clients.iter() {
            if pid::exists(pid, || {}) {
                let mut logger = nvim.logger();

                logger.set_title(format!("Compile:{name}"));
                logger.set_running(false).await.ok();

                logger.open_win().await.ok();
                logger.append(err.to_string()).await.ok();

                logger.set_status_end(false, true).await.ok();
            }
        }

        return Err(err);
    }

    Ok(true)
}
