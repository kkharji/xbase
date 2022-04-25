// See https://clang.llvm.org/docs/JSONCompilationDatabase.html
// See https://github.com/apple/sourcekit-lsp/blob/main/Sources/SKCore/CompilationDatabase.swift

use serde::{Deserialize, Serialize};

use crate::util::regex::matches_compile_swift_sources;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompileCommand {
    /// Module name
    /// NOTE: not sure if this required
    #[serde(
        rename(serialize = "module_name"),
        skip_serializing_if = "String::is_empty"
    )]
    pub name: String,

    /// The path of the main file for the compilation, which may be relative to `directory`.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub file: String,

    /// The wroking directory for the compilation
    pub directory: String,

    /// The compile command, this is alias with commandLine or split form of command
    pub command: String,

    /// Source code files.
    #[serde(rename(serialize = "fileLists"), skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<String>,

    /// For SwiftFileList
    #[serde(rename(serialize = "fileLists"))]
    pub file_lists: Vec<String>,

    /// The name of the build output
    #[serde(skip_serializing_if = "String::is_empty")]
    pub output: String,

    /// Index store path. Kept for the caller to further process.
    #[serde(skip)]
    pub index_store_path: Option<String>,
}

impl CompileCommand {
    pub fn can_parse(line: &String) -> bool {
        matches_compile_swift_sources(line)
    }

    /// Parse starting from current line as swift module
    /// Matching r"^CompileSwiftSources\s*"
    pub fn swift_module(lines: &Vec<String>, cursor: usize) -> Option<CompileCommand> {
        let directory = match lines.get(cursor) {
            Some(s) => s.trim().replace("cd ", ""),
            None => {
                tracing::error!("Found COMPILE_SWIFT_MODULE_PATERN but no more lines");
                return None;
            }
        };

        let command = match lines.get(cursor) {
            Some(s) => s.trim().to_string(),
            None => {
                tracing::error!("Found COMPILE_SWIFT_MODULE_PATERN but couldn't extract command");
                return None;
            }
        };

        let arguments = match shell_words::split(&command) {
            Ok(l) => l,
            Err(e) => {
                tracing::error!("Fail to create swift module command {e}");
                return None;
            }
        };

        // NOTE: This is never changed
        let file = String::default();
        let output = String::default();
        let mut name = String::default();
        let mut files = Vec::default();
        let mut file_lists = Vec::default();
        let mut index_store_path = None;

        for i in 0..arguments.len() {
            let val = &arguments[i];
            if val == "-module-name" {
                name = arguments[i + 1].to_owned();
            } else if val == "-index-store-path" {
                index_store_path = Some(arguments[i + 1].to_owned());
            } else if val.ends_with(".swift") {
                files.push(val.to_owned());
            } else if val.ends_with(".SwiftFileList") {
                file_lists.push(val.replace("@", "").to_owned());
            }
        }

        let command = Self {
            directory,
            command: arguments.join(" "),
            name,
            file,
            output,
            files,
            file_lists,
            index_store_path,
        };

        tracing::debug!("Got Swift commands for {}", command.directory);
        tracing::trace!("{:#?}", command);
        Some(command)
    }
}
