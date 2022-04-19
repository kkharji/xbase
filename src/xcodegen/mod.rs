use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;

use anyhow::{Context, Result};

use crate::Workspace;

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
pub fn config_path(ws: &Workspace) -> PathBuf {
    /*
    TODO: support otherways to identify xcodegen project

    Some would have xcodegen config as json file or
    have different location to where they store xcodegen project config.
    */
    ws.root.join("project.yml")
}

/// Checks whether current workspace is xcodegen project.
pub fn is_workspace(ws: &Workspace) -> bool {
    crate::xcodegen::config_path(ws).exists()
}
