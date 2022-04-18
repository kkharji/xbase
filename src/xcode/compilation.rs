// CREDIT: @SolaWing https://github.com/SolaWing/xcode-build-server/blob/master/xcode-build-server
// CREDIT: Richard Howell https://github.com/doc22940/sourcekit-lsp/blob/master/Tests/INPUTS/BuildServerBuildSystemTests.testBuildTargetOutputs/server.py

#[cfg(feature = "serial")]
mod command;
use anyhow::Result;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

// TODO: Support compiling commands for objective-c files
// TODO: Test multiple module command compile
// TODO: support index store

pub struct Compiliation {
    pub commands: Vec<command::CompilationCommand>,
    lines: Vec<String>,
    clnum: usize,
    index_store_path: Vec<String>,
}

impl Compiliation {
    pub fn new(build_log: Vec<String>) -> Self {
        let mut parser = Self {
            lines: build_log,
            clnum: 0,
            commands: Vec::default(),
            index_store_path: Vec::default(),
        };

        for line in parser.lines.iter() {
            parser.clnum += 1;

            if line.starts_with("===") {
                continue;
            }

            if RE["swift_module"].is_match(line) {
                if let Some(command) = parser.swift_module_command() {
                    if let Some(isp) = &command.index_store_path {
                        parser.index_store_path.push(isp.clone())
                    }
                    parser.commands.push(command);
                    continue;
                };
            }
        }
        parser
    }

    /// Serialize to JSON string
    #[cfg(feature = "serial")]
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.commands)
    }
}

lazy_static! {
    static ref RE: HashMap<&'static str, Regex> = HashMap::from([
        (
            "swift_module",
            Regex::new(r"^CompileSwiftSources\s*").unwrap()
        ),
        (
            "swift",
            Regex::new(r"^CompileSwift\s+ \w+\s+ \w+\s+ (.+)$").unwrap()
        )
    ]);
}

impl Compiliation {
    /// Parse starting from current line as swift module
    /// Matching r"^CompileSwiftSources\s*"
    fn swift_module_command(&self) -> Option<command::CompilationCommand> {
        let directory = match self.lines.get(self.clnum) {
            Some(s) => s.trim().replace("cd ", ""),
            None => {
                tracing::error!("Found COMPILE_SWIFT_MODULE_PATERN but no more lines");
                return None;
            }
        };

        let command = match self.lines.get(self.clnum + 3) {
            Some(s) => s.trim().to_string(),
            None => {
                tracing::error!("Found COMPILE_SWIFT_MODULE_PATERN but couldn't extract command");
                return None;
            }
        };

        match command::CompilationCommand::new(directory, command) {
            Ok(command) => {
                tracing::debug!("Extracted {} Module Command", command.name);
                Some(command)
            }
            Err(e) => {
                tracing::error!("Fail to create swift module command {e}");
                None
            }
        }
    }

    /// Parse starting from current line as swift module
    /// Matching "^CompileSwift\s+ \w+\s+ \w+\s+ (.+)$"
    #[allow(dead_code)]
    fn swift_command(&self, _line: &str) {}
}

#[test]
fn test() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let build_log_test = tokio::fs::read_to_string("/Users/tami5/repos/swift/wordle/build.log")
            .await
            .unwrap()
            .split("\n")
            .map(|l| l.to_string())
            .collect();
        let compiliation = Compiliation::new(build_log_test);

        println!("{}", compiliation.to_json().unwrap())
    });
}
