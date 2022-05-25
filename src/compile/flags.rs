use super::CompilationCommand;
use crate::error::{ConversionError, EnsureOptional};
use crate::util::fs::{find_header_dirs, find_swift_files, find_swift_module_root, get_files_list};
use crate::Result;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::read_to_string,
    path::{Path, PathBuf},
};
use tap::{Pipe, Tap};

const SDKPATH: &str = "/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk/";

/// File Compilation Flags
///
/// Primarily used This is used in [`crate::server::BuildServer`] to support completion and code
/// navigation for workspace files.
#[derive(Debug, Clone, Serialize, Deserialize, derive_deref_rs::Deref)]
pub struct CompileFlags(Vec<String>);

impl CompileFlags {
    /// Generate compile flags from [`crate::compile::CompilationCommand`].command.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let compilation_command;
    /// CompileFlags::from_command(&compilation_command.command);
    /// ```
    #[tracing::instrument(skip_all, level = "trace")]
    pub fn from_command(command: &str) -> Result<Self> {
        command
            .pipe(shell_words::split)?
            .tap_mut(|flags| {
                flags.remove(0);
            })
            .pipe(|flags| Self::with_files_list_content(flags))?
            .pipe(Self)
            .pipe(Result::Ok)
    }

    /// Generate compile flags from filepath.
    ///
    /// This in case a [`crate::compile::CompilationCommand`] can't be generated and only filepath
    /// is available
    /// # Examples
    ///
    /// ```no_run
    /// CompileFlags::from_filepath("/path/to/project/src/file");
    /// ```
    #[tracing::instrument(skip_all, level = "trace")]
    pub fn from_filepath(filepath: &Path) -> Result<Self> {
        let (ref project_root, swiftflags_filepath, compile_filepath) =
            find_swift_module_root(filepath);
        if let Some(project_root) = project_root {
            if let Some(ref compile_filepath) = compile_filepath {
                return super::parse_from_file(compile_filepath)?
                    .iter()
                    .flat_map(CompilationCommand::compile_flags)
                    .flatten()
                    .collect::<HashMap<_, _>>()
                    .get(filepath)
                    .to_result("CompileFileFlags", filepath)?
                    .clone()
                    .pipe(Result::Ok);
            } else if let Some(ref swiftflags_filepath) = swiftflags_filepath {
                let mut flags_collect = Vec::default();
                let (headers, frameworks) = find_header_dirs(project_root)?;

                headers
                    .into_iter()
                    .flat_map(|header| vec!["-Xcc".into(), "-I".into(), header])
                    .collect::<Vec<String>>()
                    .pipe_ref_mut(|flags| flags_collect.append(flags));

                frameworks
                    .into_iter()
                    .map(|framework| format!("-F{framework}"))
                    .collect::<Vec<String>>()
                    .pipe_ref_mut(|flags| flags_collect.append(flags));

                find_swift_files(project_root)?.pipe_ref_mut(|flags| flags_collect.append(flags));

                if let Some(ref mut additional_flags) = additional_flags(swiftflags_filepath) {
                    flags_collect.append(additional_flags)
                }

                return flags_collect.pipe(Self).pipe(Result::Ok);
            };
        }

        filepath
            .to_str()
            .ok_or_else(|| ConversionError::PathToString(filepath.into()))?
            .pipe(|f| vec![f.into(), "-sdk".into(), SDKPATH.into()])
            .pipe(Self)
            .pipe(Result::Ok)
    }

    /// Filter swift compilation arguments and inject files_list content to arguments
    pub fn with_files_list_content(flags: Vec<String>) -> Result<Vec<String>> {
        let mut args = vec![];
        let mut items = flags.into_iter();
        while let Some(arg) = items.next() {
            // sourcekit dont support filelist, unfold it
            if arg == "-filelist" {
                if let Some(arg) = items.next() {
                    arg.pipe(PathBuf::from)
                        .pipe(get_files_list)?
                        .pipe_as_mut(|paths| args.append(paths));
                    continue;
                }
            }

            // swift 5.1 filelist, unfold it
            if arg.starts_with("@") {
                if let Some(arg) = arg.strip_prefix("@") {
                    arg.pipe(get_files_list)?
                        .pipe_as_mut(|paths| args.append(paths));
                    continue;
                }
            }

            args.push(arg)
        }

        Ok(args)
    }
}

/// Get Additional flags from an optional flags_path.
fn additional_flags(flags_path: &Path) -> Option<Vec<String>> {
    read_to_string(flags_path)
        .ok()?
        .split("\n")
        .filter(|line| line.starts_with("#"))
        .map(|line| line.trim().to_string())
        .collect::<Vec<_>>()
        .into()
}
