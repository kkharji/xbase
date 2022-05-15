pub const DAEMON_SOCKET_PATH: &str = "/tmp/xbase-daemon.socket";
pub const DAEMON_BINARY: &str =
    "/Users/tami5/repos/neovim/xbase.nvim/target/debug/xbase-daemon";

#[cfg(feature = "daemon")]
pub type DaemonSharedState = std::sync::Arc<tokio::sync::Mutex<crate::state::State>>;

#[cfg(feature = "server")]
pub type ServerSharedState = std::sync::Arc<std::sync::Mutex<crate::state::State>>;

#[cfg(feature = "daemon")]
lazy_static::lazy_static! {
    pub static ref DAEMON_STATE: DaemonSharedState = {
        use crate::state::State;
        use std::sync::Arc;
        use tokio::sync::Mutex;

        Arc::new(Mutex::new(State {
            #[cfg(feature = "server")]
            compile_commands: Default::default(),
            #[cfg(feature = "server")]
            file_flags: Default::default(),
            #[cfg(feature = "daemon")]
            projects: Default::default(),
            #[cfg(feature = "daemon")]
            clients: Default::default(),
            #[cfg(feature = "daemon")]
            watcher: Default::default(),
        }))

    };

}

#[cfg(feature = "server")]
lazy_static::lazy_static! {
    pub static ref SERVER_STATE: ServerSharedState = {
        use crate::state::State;
        use std::sync::{Arc, Mutex};

        Arc::new(Mutex::new(State {
            #[cfg(feature = "server")]
            compile_commands: Default::default(),
            #[cfg(feature = "server")]
            file_flags: Default::default(),
            #[cfg(feature = "daemon")]
            projects: Default::default(),
            #[cfg(feature = "daemon")]
            clients: Default::default(),
            #[cfg(feature = "daemon")]
            watcher: Default::default(),
        }))

    };
}
