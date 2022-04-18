use anyhow::Result;
use std::ffi;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::process::{ExitStatus, Stdio};
use tokio::process::Command;

use super::compilation::Compiliation;
use serde_json::json;
use tokio::fs;
use tokio::io::AsyncWriteExt;

// https://github.com/Gordon-F/cargo-xcodebuild
/// run xcodebuild build with extra arguments
pub async fn build<P, I, S>(root: P, args: I) -> Result<Vec<String>>
where
    P: AsRef<Path> + Debug,
    I: IntoIterator<Item = S>,
    S: AsRef<ffi::OsStr>,
{
    tracing::debug!("Building {:?}", root);
    let output = Command::new("/usr/bin/xcodebuild")
        .arg("build")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .current_dir(root)
        .spawn()?
        .wait_with_output()
        .await
        .map(|o| String::from_utf8(o.stdout))??
        .split("\n")
        .map(|s| s.to_string())
        .collect();

    tracing::trace!(
        "xcodebuild output: \n{:#?}\n\n\n---------------------------------- end",
        output
    );
    Ok(output)
}

/// run xcodebuild clean with extra arguments
pub async fn clean<P, I, S>(root: P, args: I) -> Result<ExitStatus, std::io::Error>
where
    P: AsRef<Path> + Debug,
    I: IntoIterator<Item = S>,
    S: AsRef<ffi::OsStr>,
{
    tracing::debug!("Cleaning {:?}", root);

    Command::new("/usr/bin/xcodebuild")
        .arg("clean")
        .args(args)
        .current_dir(root)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start xcodebuild clean")
        .wait()
        .await
}

#[cfg(feature = "serial")]
pub async fn update_compiled_commands(root: &PathBuf, build_log: Vec<String>) -> Result<()> {
    fs::write(
        root.join(".compile"),
        Compiliation::new(build_log).to_json()?,
    )
    .await?;
    tracing::info!("Updated Compiled Commands");
    Ok(())
}

pub async fn ensure_server_config_file(root: &PathBuf) -> Result<()> {
    let path = root.join("buildServer.json");
    if fs::File::open(&path).await.is_ok() {
        tracing::debug!("buildServer.json exists.");
        return Ok(());
    }

    let mut file = fs::File::create(path).await?;
    let config = json! ({
        "name": "XcodeBase Server",
        // FIXME: Point to user xcode-build-server
        "argv": ["/Users/tami5/repos/neovim/XcodeBase.nvim/target/debug/xcodebase-build-server"],
        "version": "0.1",
        "bspVersion": "0.2",
        "languages": [
            "swift",
            "objective-c",
            "objective-cpp",
            "c",
            "cpp"
        ],
    });

    file.write_all(config.to_string().as_ref()).await?;
    file.sync_all().await?;
    file.shutdown().await?;

    tracing::debug!("buildServer.json created!");

    Ok(())
}
