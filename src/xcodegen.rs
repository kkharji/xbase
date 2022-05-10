//! Helper functions to communicate with xcodegen
use crate::daemon::Workspace;
use anyhow::{Context, Result};
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
    xcodgen()
        .current_dir(root)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .arg("generate")
        .arg("-c")
        .spawn()?
        .wait()
        .await
        .context("xcodegen generate")
}

// NOTE: passing workspace in-case in the future we would allow configurability of project.yml path
pub fn get_config_path(ws: &Workspace) -> PathBuf {
    /*
    TODO: support otherways to identify xcodegen project

    Some would have xcodegen config as json file or
    have different location to where they store xcodegen project config.
    */
    ws.root.join("project.yml")
}

/// Checks whether current workspace is xcodegen project.
pub fn is_valid(ws: &Workspace) -> bool {
    crate::xcodegen::get_config_path(ws).exists()
}

pub async fn regenerate(name: &str, path: PathBuf, root: &PathBuf) -> Result<bool> {
    if !root.join("project.yml").exists() {
        anyhow::bail!("Project.yml is not found");
    }

    tracing::info!("Updating {name}.xcodeproj");

    let mut retry_count = 0;
    while retry_count < 3 {
        if let Ok(code) = generate(&root).await {
            if code.success() {
                if path
                    .file_name()
                    .ok_or_else(|| anyhow::anyhow!("Fail to get filename from {:?}", path))?
                    .eq("project.yml")
                {
                    return Ok(true);
                }
                return Ok(false);
            }
        }
        retry_count += 1
    }

    anyhow::bail!("Fail to update_xcodeproj")
}
