//! Module for generating Compilation Database.
//!
//! based on <https://clang.llvm.org/docs/JSONCompilationDatabase.html>
//!
//! see <https://github.com/apple/sourcekit-lsp/blob/main/Sources/SKCore/CompilationDatabase.swift>
mod command;
mod flags;

use anyhow::{Context, Result};
pub use command::CompilationCommand;
pub use flags::CompileFlags;
use serde::{Deserialize, Serialize};
use std::{ops::Deref, path};
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
#[derive(Debug, Deserialize, Serialize)]
pub struct CompilationDatabase(pub Vec<CompilationCommand>);

impl IntoIterator for CompilationDatabase {
    type Item = CompilationCommand;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Deref for CompilationDatabase {
    type Target = Vec<CompilationCommand>;

    fn deref(&self) -> &Self::Target {
        &self.0
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
        .pipe_ref(|s| serde_json::from_str(s))
        .context("Deserialize .compile")
}

/// Generate [`CompilationDatabase`] from xcodebuild::parser::Step
///
pub async fn generate_from_steps(steps: &Vec<Step>) -> Result<CompilationDatabase> {
    let mut steps = steps.iter();
    let mut _index_store_path = Vec::default();
    let mut commands = vec![];

    while let Some(step) = steps.next() {
        if let Step::CompileSwiftSources(sources) = step {
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
    tracing::debug!("Generated compilation database from logs");
    Ok(CompilationDatabase(commands))
}

#[cfg(feature = "daemon")]
pub async fn update_compilation_file(root: &path::PathBuf) -> Result<()> {
    use crate::xcode::fresh_build;
    use tokio_stream::StreamExt;

    // TODO(build): Ensure that build successed. check for Exit Code
    let steps = fresh_build(&root).await?.collect::<Vec<Step>>().await;
    let compile_commands = steps.pipe_ref(generate_from_steps).await?;
    if compile_commands.is_empty() {
        let msg = format!("No compile commands generated\n{:#?}", steps);
        anyhow::bail!("{msg}")
    }

    let json = serde_json::to_vec_pretty(&compile_commands)?;
    tokio::fs::write(root.join(".compile"), &json).await?;

    Ok(())
}

/// Ensure that buildServer.json exists in root directory.
#[cfg(feature = "daemon")]
pub async fn ensure_server_config(root: &path::PathBuf) -> Result<()> {
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;

    let path = root.join("buildServer.json");
    if tokio::fs::File::open(&path).await.is_ok() {
        return Ok(());
    }

    tracing::info!("Creating {:?}", path);

    let mut file = tokio::fs::File::create(path).await?;
    let config = serde_json::json! ({
        "name": "xbase Server",
        // FIXME: Point to user xcode-build-server
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
