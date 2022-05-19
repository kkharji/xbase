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

pub fn string_as_section(content: String) -> String {
    let empty_string = content.is_empty();
    let mut content = format!("[{content}");
    let len = content.len();
    let rep = 73 - len;
    if rep > 0 {
        let sep = "-".repeat(rep);
        if !empty_string {
            content.push_str(" ");
        } else {
            content.push_str("-");
        }
        content.push_str(&sep);
    }
    content.push_str("]");
    content
}
