mod command;
mod flags;
pub use command::CompileCommand;
pub use flags::CompileFlags;

use crate::util::regex::matches_compile_swift_sources;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::{ops::Deref, path::Path};
use tap::Pipe;

// TODO: Support compiling commands for objective-c files
// TODO: Test multiple module command compile

#[derive(Debug, Deserialize)]
pub struct CompileCommands(pub Vec<CompileCommand>);

impl IntoIterator for CompileCommands {
    type Item = CompileCommand;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Deref for CompileCommands {
    type Target = Vec<CompileCommand>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl CompileCommands {
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
                if let Some(command) = CompileCommand::swift_module(&lines, cursor) {
                    if let Some(ref index_store_path) = command.index_store_path {
                        _index_store_path.push(index_store_path.clone());
                    }
                    commands.push(command);
                }
            }
        }

        Self(commands)
    }

    pub fn from_file(path: &Path) -> Result<Self> {
        std::fs::read_to_string(path)?
            .pipe_ref(|s| serde_json::from_str(s))
            .context("Deserialize .compile")
    }

    /// Generate and write compile commands from build logs to directory
    #[cfg(feature = "async")]
    pub async fn update(dir: &std::path::PathBuf, build_log: Vec<String>) -> Result<()> {
        tracing::info!("Updating .compile in {:?}", dir);
        Self::from_logs(build_log)
            .pipe(|cmd| serde_json::to_vec_pretty(&cmd.0))?
            .pipe(|json| tokio::fs::write(dir.join(".compile"), json))
            .await
            .context("Write CompileCommands")
    }
}

#[test]
fn test() {
    #[cfg(feature = "async")]
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        use tap::Pipe;
        tokio::fs::read_to_string("/Users/tami5/repos/swift/wordle/build.log")
            .await
            .unwrap()
            .split("\n")
            .map(|l| l.to_string())
            .collect::<Vec<_>>()
            .pipe(CompileCommands::from_logs)
            .pipe(|v| println!("{:#?}", v));
    });
}
