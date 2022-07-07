use crate::broadcast::Broadcast;
use crate::runtime;
use std::ffi::CStr;
use std::ops::Deref;
// use safer_ffi::layout::ReprC;
// use safer_ffi::prelude::*;
use std::path::PathBuf;
use tap::Pipe;
use xbase_proto::*;

/// Project root registration response
#[repr(C)]
pub struct RegisterResponse {
    status: RegisterStatus,
    fd: i32,
}
// TODO: Provide an interface for xbase_proto errors
// see https://rust-unofficial.github.io/patterns/idioms/ffi/errors.html
#[repr(u8)]
/// Project root registration Status
pub enum RegisterStatus {
    /// Project root is registered successfully
    Registered,
    /// Project root is not supported
    NotSupported,
    /// Failed to setup broadcast writer
    BroadcastWriterSetupErrored,
    /// Server returned an Error
    ServerErrored,
}

// #[ffi_export]
#[no_mangle]
/// Register given root in xbase-daemon.
///
/// - Takes `root` of project; checks whether it the project root is supported.
/// - Ensures that xbase-daemon is running.
/// - Sends register request to the daemon.
/// - Subscribes client to project root logging broadcast socket.
///   NOTE: if the client is already subscribed, the function will return the cached raw_fd
/// - Returns `RegisterResponse` with status and file description to read broadcast messages.
pub extern "C" fn xbase_register(root: *const libc::c_char) -> RegisterResponse {
    let root = unsafe { CStr::from_ptr(root) }
        .to_string_lossy()
        .deref()
        .pipe(PathBuf::from);

    // NOTE: Should skip for previously registered path?
    if !(root.join("project.yml").exists()
        || root.join("Project.swift").exists()
        || root.join("Package.swift").exists()
        || wax::walk("*.xcodeproj", &root)
            .map(|w| w.count() != 0)
            .unwrap_or_default())
    {
        return RegisterResponse {
            status: RegisterStatus::NotSupported,
            fd: 0,
        };
    }

    runtime::ensure_daemon();

    let root_ = root.clone();
    let result = runtime::request!(register, root_);
    let address = match result {
        Ok(Ok(p)) => p,
        // TODO: inspect error further
        _ => {
            return RegisterResponse {
                status: RegisterStatus::ServerErrored,
                fd: 0,
            }
        }
    };
    let fd = match Broadcast::init_or_skip(address, root) {
        Ok(fd) => fd,
        Err(_) => {
            return RegisterResponse {
                status: RegisterStatus::BroadcastWriterSetupErrored,
                fd: 0,
            }
        }
    };

    RegisterResponse {
        status: RegisterStatus::Registered,
        fd,
    }
}
