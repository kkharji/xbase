pub mod broadcast;
pub mod error;
pub mod project;
mod runner;
pub mod server;
mod state;
pub mod types;
mod util;
mod watcher;

pub use {broadcast::*, error::*, project::*, runner::*, state::*, types::*, util::*, watcher::*};

pub static SOCK_ADDR: &str = "/tmp/xbase.socket";
pub static PID_PATH: &str = "/tmp/xbase.pid";
pub static LOG_PATH: &str = "/tmp/xbase.log";
