/// Where the daemon socket path will be located
pub static DAEMON_SOCKET_PATH: &str = "/tmp/xbase-daemon.socket";

/// Where the daemon pid will be located
pub static DAEMON_PID_PATH: &str = "/tmp/xbase-daemon-pid";

/// HACK: This static path would break if a pre-built binary is to be distributed.
///
/// A soultion might be to move binaries to `~/.local/share/xbase/bin`
/// Where the daemon binary will be located.
pub static DAEMON_BINARY_PATH: &'static str = {
    if cfg!(debug_assertions) {
        concat!(env!("CARGO_MANIFEST_DIR"), "/target/debug/xbase-daemon")
    } else {
        concat!(env!("CARGO_MANIFEST_DIR"), "/bin/xbase-daemon")
    }
};

/// Where the server binary will be located.
pub static SERVER_BINARY_PATH: &'static str = {
    if cfg!(debug_assertions) {
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/target/debug/xbase-sourcekit-helper"
        )
    } else {
        concat!(env!("CARGO_MANIFEST_DIR"), "/bin/xbase-sourcekit-helper")
    }
};

#[cfg(feature = "daemon")]
pub type DaemonSharedState = std::sync::Arc<tokio::sync::Mutex<crate::state::State>>;

#[cfg(feature = "daemon")]
lazy_static::lazy_static! {
    pub static ref DAEMON_STATE: DaemonSharedState = {
        use crate::state::State;
        use std::sync::Arc;
        use tokio::sync::Mutex;

        Arc::new(Mutex::new(State {
            projects: Default::default(),
            clients: Default::default(),
            watcher: Default::default(),
            devices: Default::default(),
        }))

    };

}
