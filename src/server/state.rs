use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::Result;
use tap::Pipe;

use crate::compile::{CompilationCommand, CompilationDatabase, CompileFlags};

/// Build Server State.
#[derive(Debug, Default)]
pub struct BuildServerState {
    compile_commands: HashMap<PathBuf, CompilationDatabase>,
    file_flags: HashMap<PathBuf, CompileFlags>,
}

impl BuildServerState {
    /// Get [`CompilationDatabase`] for a .compile file path.
    pub fn compile_commands(&mut self, compile_filepath: &Path) -> Result<&CompilationDatabase> {
        if self.compile_commands.contains_key(compile_filepath) {
            &self.compile_commands[compile_filepath]
        } else {
            CompilationDatabase::from_file(compile_filepath)?
                .pipe(|cmds| self.compile_commands.insert(compile_filepath.into(), cmds))
                .pipe(|_| &self.compile_commands[compile_filepath])
        }
        .pipe(Result::Ok)
    }

    /// Get [`CompileFlags`] for a file
    pub fn file_flags(
        &mut self,
        filepath: &Path,
        compile_filepath: Option<&PathBuf>,
    ) -> Result<&CompileFlags> {
        if let Some(compile_filepath) = compile_filepath {
            if self.file_flags.contains_key(filepath) {
                self.file_flags.get(filepath)
            } else {
                self.compile_commands(compile_filepath)?
                    .iter()
                    .flat_map(CompilationCommand::compile_flags)
                    .flatten()
                    .collect::<HashMap<_, _>>()
                    .pipe(|map| self.file_flags.extend(map))
                    .pipe(|_| self.file_flags.get(filepath))
            }
        } else {
            CompileFlags::from_filepath(filepath)?
                .pipe(|flags| self.file_flags.insert(filepath.to_path_buf(), flags))
                .pipe(|_| self.file_flags.get(filepath))
        }
        .ok_or_else(|| anyhow::anyhow!("Couldn't find file flags for {:?}", filepath))
    }

    /// Clear [`BuildServerState`]
    pub fn clear(&mut self) {
        self.file_flags = Default::default();
        self.compile_commands = Default::default();
    }
}
#[test]
fn test_file_flags() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    let mut state = BuildServerState::default();
    let flags = state.file_flags(
        &PathBuf::from("file:///Users/tami5/repos/swift/wordle/Source/Views/GuessView.swift"),
        Some(&PathBuf::from("/Users/tami5/repos/swift/wordle/.compile")),
    );

    tracing::info!("{:?}", flags)
}
