//! General utilities
#[cfg(any(feature = "server", feature = "daemon"))]
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
