// See https://clang.llvm.org/docs/JSONCompilationDatabase.html
// See https://github.com/apple/sourcekit-lsp/blob/main/Sources/SKCore/CompilationDatabase.swift

use anyhow::Result;
use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub struct CompilationCommand {
    /// Module name
    /// NOTE: not sure if this required
    #[serde(
        rename(serialize = "module_name"),
        skip_serializing_if = "String::is_empty"
    )]
    pub name: String,

    /// The path of the main file for the compilation, which may be relative to `directory`.
    #[serde(skip_serializing_if = "String::is_empty")]
    file: String,

    /// The wroking directory for the compilation
    directory: String,

    /// The compile command, this is alias with commandLine or split form of command
    #[serde(skip_serializing_if = "Vec::is_empty")]
    arguments: Vec<String>,

    /// Source code files.
    #[serde(rename(serialize = "fileLists"), skip_serializing_if = "Vec::is_empty")]
    files: Vec<String>,

    /// For SwiftFileList
    #[serde(rename(serialize = "fileLists"))]
    file_lists: Vec<String>,

    /// The name of the build output
    #[serde(skip_serializing_if = "String::is_empty")]
    output: String,

    /// Index store path. Kept for the caller to further process.
    #[serde(skip)]
    pub index_store_path: Option<String>,
}

impl CompilationCommand {
    pub fn new(directory: String, command: String) -> Result<Self> {
        let arguments = shell_words::split(&command)?;

        let mut module = Self {
            directory,
            arguments,
            name: String::default(),
            file: String::default(),
            output: String::default(),
            files: Vec::default(),
            file_lists: Vec::default(),
            index_store_path: None,
        };

        for i in 0..module.arguments.len() {
            let val = &module.arguments[i];
            if val == "-module-name" {
                module.name = module.arguments[i + 1].to_owned();
            } else if val == "-index-store-path" {
                module.index_store_path = Some(module.arguments[i + 1].to_owned());
            } else if val.ends_with(".swift") {
                module.files.push(val.to_owned());
            } else if val.ends_with(".SwiftFileList") {
                module.file_lists.push(val.replace("@", "").to_owned());
            }
        }

        Ok(module)
    }
}
