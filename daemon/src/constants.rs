use crate::store::*;
use std::{path::PathBuf, sync::Arc};
use tokio::sync::Mutex;

/// Where the daemon socket path will be located
pub static DAEMON_SOCKET_PATH: &str = "/tmp/xbase.socket";

/// Where the daemon pid will be located
pub static DAEMON_PID_PATH: &str = "/tmp/xbase.pid";

pub type DaemonSharedState = Arc<Mutex<State>>;

/// Build Server State.
#[derive(Default, Debug)]
pub struct State {
    /// Managed Workspaces
    pub projects: ProjectStore,
    /// Managed watchers
    pub watcher: WatchStore,
    /// Available Devices
    pub devices: Devices,
    /// Loggers
    pub broadcasters: BroadcastStore,
}

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
        Arc::new(Mutex::new(State {
            projects: Default::default(),
            watcher: Default::default(),
            devices: Default::default(),
            broadcasters: Default::default(),
        }))

    };

}
