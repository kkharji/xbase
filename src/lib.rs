pub mod broadcast;
pub mod error;
pub mod project;
mod runner;
mod runtime;
pub mod server;
pub mod types;
mod util;
mod watcher;

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::{path::PathBuf, sync::Arc};
use tokio::sync::{Mutex, OwnedMutexGuard};

pub use {
    broadcast::*, error::*, project::*, runner::*, runtime::*, types::*, util::*, watcher::*,
};

pub static SOCK_ADDR: &str = "/tmp/xbase.socket";
pub static PID_PATH: &str = "/tmp/xbase.pid";
pub static LOG_PATH: &str = "/tmp/xbase.log";
pub static BIN_ROOT: &str = "$HOME/.local/share/xbase";

pub type ProjectRuntimes = HashMap<PathBuf, PRMessageSender>;

static RUNTIMES: Lazy<Arc<Mutex<ProjectRuntimes>>> = Lazy::new(Default::default);

/// Get OwnedMutexGuard of runtimes
#[tracing::instrument(name = "Runtimes")]
pub async fn runtimes() -> OwnedMutexGuard<ProjectRuntimes> {
    tracing::trace!("Locking");
    let x = RUNTIMES.clone().lock_owned().await;
    tracing::trace!("Returning");
    x
}
