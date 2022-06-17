//! Helper functions to communicate with xcodegen
use crate::error::{ConversionError, XcodeGenError};
use crate::Result;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;

#[inline]
fn xcodgen() -> Result<tokio::process::Command> {
    let xcodegen_path = which::which("xcodegen")?;
    Ok(tokio::process::Command::new(xcodegen_path))
}

// Run xcodgen generate
pub async fn generate<P: AsRef<Path> + Debug>(root: P) -> Result<ExitStatus> {
    tracing::info!("Regenerating xcodegen project");
    let status = xcodgen()?
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

/// Checks whether current workspace is xcodegen project.
pub fn is_valid(root: &PathBuf) -> bool {
    config_file(root).is_ok()
}

pub fn config_file(root: &PathBuf) -> Result<PathBuf> {
    /*
    TODO: support otherways to identify xcodegen project

    Some would have xcodegen config as json file or
    have different location to where they store xcodegen project config.
    */
    let config_path = root.join("project.yml");
    if !config_path.exists() {
        return Err(XcodeGenError::NoProjectConfig.into());
    }
    Ok(config_path)
}
