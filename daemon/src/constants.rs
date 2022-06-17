use std::path::PathBuf;

/// Where the daemon socket path will be located
pub static DAEMON_SOCKET_PATH: &str = "/tmp/xbase-daemon.socket";

/// Where the daemon pid will be located
pub static DAEMON_PID_PATH: &str = "/tmp/xbase-daemon-pid";

pub type DaemonSharedState = std::sync::Arc<tokio::sync::Mutex<crate::state::State>>;

lazy_static::lazy_static! {
    /// Where the server binary will be located.
    pub static ref SERVER_BINARY_PATH: PathBuf = {
        let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf();
        if cfg!(debug_assertions) {
            root.extend(&["target", "debug", "xbase-sourcekit-helper"]);
        } else {
            root.extend(&["bin", "xbase-sourcekit-helper"]);
        }
        root
    };

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
