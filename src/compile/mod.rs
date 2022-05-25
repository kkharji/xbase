//! Module for generating Compilation Database.
//!
//! based on <https://clang.llvm.org/docs/JSONCompilationDatabase.html>
//!
//! see <https://github.com/apple/sourcekit-lsp/blob/main/Sources/SKCore/CompilationDatabase.swift>
mod command;
mod flags;

use crate::Result;
pub use command::CompilationCommand;
pub use flags::CompileFlags;
use serde::{Deserialize, Serialize};
use std::path;
use tap::Pipe;
use xcodebuild::parser::Step;

// TODO: Support compiling commands for objective-c files

/// A clang-compatible compilation Database
///
/// It depends on build logs generated from xcode
///
/// `xcodebuild clean -verbose && xcodebuild build`
///
/// See <https://clang.llvm.org/docs/JSONCompilationDatabase.html>
#[derive(Debug, Deserialize, Serialize, derive_deref_rs::Deref)]
pub struct CompilationDatabase(pub Vec<CompilationCommand>);

impl IntoIterator for CompilationDatabase {
    type Item = CompilationCommand;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// Parse [`CompilationDatabase`] from .compile file
///
/// Examples:
///
/// ```no_run use xbase::compile::CompilationDatabase;
/// CompilationDatabase::from_file("/path/to/xcode_build_logs");
/// ```
pub fn parse_from_file<P: AsRef<path::Path> + Clone>(path: P) -> Result<CompilationDatabase> {
    std::fs::read_to_string(path)?
        .pipe_ref(|s| serde_json::from_str::<CompilationDatabase>(s))?
        .pipe(Ok)
}

/// Generate [`CompilationDatabase`] from xcodebuild::parser::Step
///
pub async fn generate_from_steps(steps: &Vec<Step>) -> Result<CompilationDatabase> {
    let mut steps = steps.iter();
    let mut _index_store_path = Vec::default();
    let mut commands = vec![];

    while let Some(step) = steps.next() {
        if let Step::CompileSwiftSources(sources) = step {
            // HACK: Commands with files key break source kit
            // unknown argument: '-frontend'
            // unknown argument: '-primary-file'
            // unknown argument: '-primary-file'
            // unknown argument: '-primary-file'
            // unknown argument: '-emit-dependencies-path'
            // unknown argument: '-emit-reference-dependencies-path'
            // unknown argument: '-enable-objc-interop'
            // unknown argument: '-new-driver-path'
            // unknown argument: '-serialize-debugging-options'
            // unknown argument: '-enable-anonymous-context-mangled-names'
            // unknown argument: '-target-sdk-version'
            // option '-serialize-diagnostics-path' is not supported by 'swiftc'; did you mean to use 'swift'?
            if sources.command.contains("swift-frontend") {
                continue;
            }

            let arguments = shell_words::split(&sources.command)?;

            let file = Default::default();
            let output = Default::default();
            let mut name = Default::default();
            let mut files = Vec::default();
            let mut file_lists = Vec::default();
            let mut index_store_path = None;

            for i in 0..arguments.len() {
                let val = &arguments[i];
                if val == "-module-name" {
                    name = Some(arguments[i + 1].to_owned());
                } else if val == "-index-store-path" {
                    index_store_path = Some(arguments[i + 1].to_owned());
                } else if val.ends_with(".swift") {
                    files.push(val.to_owned());
                } else if val.ends_with(".SwiftFileList") {
                    file_lists.push(val.replace("@", "").to_owned());
                }
            }
            if let Some(ref index_store_path) = index_store_path {
                _index_store_path.push(index_store_path.clone());
            }

            commands.push(CompilationCommand {
                name,
                file,
                directory: sources.root.to_str().unwrap().to_string(),
                command: sources.command.clone(),
                files: files.into(),
                file_lists,
                output,
                index_store_path,
            })
        };
    }
    tracing::debug!("[Compile] regenerated compilation database");
    Ok(CompilationDatabase(commands))
}

#[cfg(feature = "daemon")]
use {
    crate::{
        client::Client,
        error::{CompileError, XcodeGenError},
        state::State,
        util::pid,
        xcode::fresh_build,
        xcodegen,
    },
    process_stream::StreamExt,
    std::path::PathBuf,
    tokio::{
        fs::{metadata, File},
        io::AsyncWriteExt,
        sync::MutexGuard,
    },
};

#[cfg(feature = "daemon")]
pub async fn update_compilation_file(root: &path::PathBuf) -> Result<()> {
    // TODO(build): Ensure that build successed. check for Exit Code
    let steps = fresh_build(&root).await?.collect::<Vec<Step>>().await;
    let compile_commands = steps.pipe_ref(generate_from_steps).await?;

    if compile_commands.is_empty() {
        return Err(CompileError::NoCompileCommandsGenerated(steps).into());
    }

    let json = serde_json::to_vec_pretty(&compile_commands)?;
    tokio::fs::write(root.join(".compile"), &json).await?;

    Ok(())
}

/// Ensure that buildServer.json exists in root directory.
#[cfg(feature = "daemon")]
pub async fn ensure_server_config(root: &path::PathBuf) -> Result<()> {
    let path = root.join("buildServer.json");
    if tokio::fs::File::open(&path).await.is_ok() {
        return Ok(());
    }

    tracing::info!("Creating {:?}", path);

    let mut file = tokio::fs::File::create(path).await?;
    let config = serde_json::json! ({
        "name": "Xbase",
        // FIXME: Point to user xbase-server
        "argv": ["/Users/tami5/repos/neovim/xbase.nvim/target/debug/xbase-server"],
        "version": "0.1",
        "bspVersion": "0.2",
        "languages": [
            "swift",
            "objective-c",
            "objective-cpp",
            "c",
            "cpp"
        ]
    });

    AsyncWriteExt::write_all(&mut file, config.to_string().as_ref()).await?;
    File::sync_all(&file).await?;
    AsyncWriteExt::shutdown(&mut file).await?;

    Ok(())
}

#[cfg(feature = "daemon")]
pub async fn ensure_server_support<'a>(
    state: &'a mut MutexGuard<'_, State>,
    client: &Client,
    path: Option<&PathBuf>,
) -> Result<bool> {
    let Client { root, .. } = client;
    let ref name = client.abbrev_root();

    let compile_path = root.join(".compile");
    let compile_exists = metadata(compile_path).await.is_ok();

    if ensure_server_config(&root).await.is_err() {
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

    if let Err(err) = update_compilation_file(&root).await {
        "setup: fail to regenerate compilation database!"
            .pipe(|msg| state.clients.echo_err(&root, &name, msg))
            .await;

        for (pid, nvim) in state.clients.iter() {
            if pid::exists(pid, || {}) {
                let mut logger = nvim.logger();

                logger.set_title(format!("Compile:{name}"));
                logger.set_running().await.ok();

                logger.open_win().await.ok();
                logger.log(err.to_string()).await.ok();

                logger.set_status_end(false, true).await.ok();
            }
        }

        return Err(err);
    }

    Ok(true)
}
