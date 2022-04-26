//! Wrapping functions for managing processes
pub fn exists(pid: &i32, cb: impl FnOnce()) -> bool {
    if libproc::libproc::proc_pid::name(*pid).is_err() {
        cb();
        false
    } else {
        true
    }
}
