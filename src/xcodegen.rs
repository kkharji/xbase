//! Helper functions to communicate with xcodegen
use crate::error::{ConversionError, XcodeGenError};
use crate::Result;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;

/*
   FIXME: make xCodeGen binary path configurable.

   Current implementation will not work unless the user has xcodeGen located in
   `~/.mint/bin/xcodegen`. Should either make it configurable as well as support a
   number of paths by default.
*/
lazy_static::lazy_static! {
    static ref XCODEGEN: PathBuf = dirs::home_dir().unwrap().join(".mint/bin/xcodegen");
}

#[inline]
fn xcodgen() -> tokio::process::Command {
    tokio::process::Command::new(&*XCODEGEN)
}

// Run xcodgen generate
pub async fn generate<P: AsRef<Path> + Debug>(root: P) -> Result<ExitStatus> {
    let status = xcodgen()
        .current_dir(root)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .arg("generate")
        .arg("-c")
        .spawn()?
        .wait()
        .await?;
    Ok(status)
}

// NOTE: passing workspace in-case in the future we would allow configurability of project.yml path
pub fn get_config_path(root: &PathBuf) -> PathBuf {
    /*
    TODO: support otherways to identify xcodegen project

    Some would have xcodegen config as json file or
    have different location to where they store xcodegen project config.
    */
    root.join("project.yml")
}

/// Checks whether current workspace is xcodegen project.
pub fn is_valid(root: &PathBuf) -> bool {
    get_config_path(root).exists()
}

pub async fn regenerate(path: &PathBuf, root: &PathBuf) -> Result<bool> {
    config_file(root)?;

    let path_to_filename_err = |path: &PathBuf| ConversionError::PathToFilename(path.into());

    let mut retry_count = 0;
    while retry_count < 3 {
        if let Ok(code) = generate(&root).await {
            if code.success() {
                if path
                    .file_name()
                    .ok_or_else(|| ConversionError::PathToFilename(path.into()))?
                    .eq("project.yml")
                {
                    return Ok(true);
                }
                return Ok(false);
            }
        }
        retry_count += 1
    }

    let project_name = root
        .file_name()
        .ok_or_else(|| path_to_filename_err(root))?
        .to_str()
        .ok_or_else(|| path_to_filename_err(root))?
        .to_string();

    Err(XcodeGenError::XcodeProjUpdate(project_name).into())
}

pub fn config_file(root: &PathBuf) -> Result<PathBuf> {
    let config_path = root.join("project.yml");
    if !config_path.exists() {
        return Err(XcodeGenError::NoProjectConfig.into());
    }
    Ok(config_path)
}
