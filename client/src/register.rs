use crate::broadcast::Broadcast;
use crate::runtime;
use safer_ffi::layout::ReprC;
use safer_ffi::prelude::*;
use std::path::PathBuf;
use tap::Pipe;
use xbase_proto::*;

#[ffi_export]
#[derive_ReprC]
#[repr(C)]
pub struct RegisterResponse {
    status: RegisterStatus,
    fd: i32,
}

// #[allow(incorrect_idnt_case)]
#[ffi_export]
#[derive_ReprC]
#[repr(u8)]
pub enum RegisterStatus {
    // Project root is registered successfully
    Registered,
    // Project root is not supported
    NotSupported,
    // Failed to setup broadcast writer
    BroadcastWriterSetupErrored,
    // Server returned an Error (TODO: the error must be known)
    ServerErrored,
}

#[ffi_export]
fn xbase_register(root: char_p::Ref<'_>) -> RegisterResponse {
    let root = root.to_str().pipe(PathBuf::from);

    // TODO: Skip for previously registered path
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
