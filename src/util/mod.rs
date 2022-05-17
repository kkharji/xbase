//! General utilities
pub mod fs;
#[cfg(feature = "lua")]
pub mod mlua;
#[cfg(feature = "serial")]
pub mod serde;
#[cfg(feature = "logging")]
pub mod tracing;

#[cfg(feature = "proc")]
/// check if process exists
pub fn proc_exists(pid: &i32, cb: impl FnOnce()) -> bool {
    if libproc::libproc::proc_pid::name(*pid).is_err() {
        cb();
        false
    } else {
        true
    }
}

#[cfg(feature = "daemon")]
pub async fn proc_kill(pid_str: &String) -> anyhow::Result<bool> {
    Ok(tokio::process::Command::new("kill")
        .arg("-9")
        .arg(pid_str)
        .output()
        .await?
        .status
        .success())
}

pub fn string_as_section(mut content: String) -> String {
    let len = content.len();
    let sep = "-".repeat(73 - len);
    content.push_str(" ");
    content.push_str(&sep);
    content
}
