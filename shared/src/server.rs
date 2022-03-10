use crate::constants::{DAEMON_BINARY, DAEMON_SOCKET_PATH};
use anyhow::Result;
use std::io::Write;
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use std::process::Command;

/// Check if Unix socket is running
pub fn is_running() -> bool {
    match UnixStream::connect(DAEMON_SOCKET_PATH) {
        Ok(s) => s.shutdown(Shutdown::Both).ok().is_some(),
        Err(_) => false,
    }
}

/// Spawn new instance of the server via running binaray is a child process
pub fn spawn() -> Result<()> {
    Command::new(DAEMON_BINARY).spawn()?;
    Ok(())
}

/// Execute argument in the runtime
pub fn execute(args: &[&str]) -> Result<()> {
    let mut stream = UnixStream::connect(DAEMON_SOCKET_PATH)?;
    stream.write_all(args.join(" ").as_str().as_ref())?;
    stream.flush()?;
    Ok(())
}
