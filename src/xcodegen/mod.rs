use std::fmt::Debug;
use std::path::{Path, PathBuf};

use anyhow::Result;

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
pub async fn generate<P: AsRef<Path> + Debug>(root: P) -> Result<Vec<String>> {
    let output = xcodgen()
        .current_dir(root)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .arg("generate")
        .spawn()
        .expect("Failed to start xcodeGen.")
        .wait_with_output()
        .await
        .expect("Failed to run xcodeGen.");

    if output.status.code().unwrap().ne(&0) {
        anyhow::bail!(
            "{:#?}",
            String::from_utf8(output.stderr)?
                .split("\n")
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
        )
    } else {
        Ok(String::from_utf8(output.stdout)?
            .split("\n")
            .map(|s| s.to_string())
            .collect())
    }
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
