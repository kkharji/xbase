//! Module for generating Compilation Database.
//!
//! based on <https://clang.llvm.org/docs/JSONCompilationDatabase.html>
//!
//! see <https://github.com/apple/sourcekit-lsp/blob/main/Sources/SKCore/CompilationDatabase.swift>
mod command;
mod flags;
pub use command::CompilationCommand;
pub use flags::CompileFlags;

use crate::util::regex::matches_compile_swift_sources;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::{ops::Deref, path::Path};
use tap::Pipe;

// TODO: Support compiling commands for objective-c files
// TODO: Test multiple module command compile

/// A clang-compatible compilation Database
///
/// It depends on build logs generated from xcode
///
/// `xcodebuild clean -verbose && xcodebuild build`
///
/// See <https://clang.llvm.org/docs/JSONCompilationDatabase.html>
#[derive(Debug, Deserialize)]
pub struct CompilationDatabase(pub Vec<CompilationCommand>);

impl CompilationDatabase {
    /// Generate [`CompilationDatabase`] from build logs.
    ///
    /// Examples:
    ///
    /// ```no_run
    /// use xcodebase::compile::CompilationDatabase;
    /// use std::fs::read_to_string;
    ///
    /// CompilationDatabase::from_logs(
    ///     read_to_string("/path/to/xcode_build_logs")
    ///            .unwrap()
    ///            .split("\n")
    ///            .map(|l| l.to_string())
    ///            .collect::<Vec<_>>()
    /// )
    /// ```
    pub fn from_logs(lines: Vec<String>) -> Self {
        // TODO: support index store
        let mut _index_store_path = Vec::default();
        let mut commands = vec![];
        let mut cursor = 0;

        for line in lines.iter() {
            cursor += 1;
            if !line.starts_with("===") {
                continue;
            }

            if matches_compile_swift_sources(line) {
                if let Some(command) = CompilationCommand::swift_module(&lines, cursor) {
                    if let Some(ref index_store_path) = command.index_store_path {
                        _index_store_path.push(index_store_path.clone());
                    }
                    commands.push(command);
                }
            }
        }

        Self(commands)
    }

    /// Generate [`CompilationDatabase`] from file path.
    ///
    /// Examples:
    ///
    /// ```no_run
    /// use xcodebase::compile::CompilationDatabase;
    ///
    /// CompilationDatabase::from_file("/path/to/xcode_build_logs");
    /// ```
    pub fn from_file(path: &Path) -> Result<Self> {
        std::fs::read_to_string(path)?
            .pipe_ref(|s| serde_json::from_str(s))
            .context("Deserialize .compile")
    }
}

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
