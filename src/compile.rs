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
use serde::Deserialize;
use std::{ops::Deref, path::Path};
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
#[derive(Debug, Deserialize)]
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

impl CompilationDatabase {
    /// Parse [`CompilationDatabase`] from .compile file
    ///
    /// Examples:
    ///
    /// ```no_run
    /// use xcodebase::compile::CompilationDatabase;
    ///
    /// CompilationDatabase::from_file("/path/to/xcode_build_logs");
    /// ```
    pub fn parse_from_file<P: AsRef<Path> + Clone>(path: P) -> Result<Self> {
        std::fs::read_to_string(path)?
            .pipe_ref(|s| serde_json::from_str(s))
            .context("Deserialize .compile")
    }

    /// Generate [`CompilationDatabase`] from xcodebuild::parser::Step
    ///
    pub async fn generate_from_steps(steps: &Vec<Step>) -> Result<Self> {
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
        Ok(Self(commands))
    }
}
