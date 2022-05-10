pub use serde::{Deserialize, Serialize};

#[cfg(feature = "daemon")]
pub use crate::daemon::Workspace;
#[cfg(feature = "daemon")]
pub use crate::state::DaemonState;
#[cfg(feature = "daemon")]
pub use anyhow::{Context, Result};
#[cfg(feature = "daemon")]
use nvim_rs::compat::tokio::Compat;
#[cfg(feature = "daemon")]
pub use nvim_rs::create::tokio::new_path as connect;
#[cfg(feature = "daemon")]
pub use nvim_rs::rpc::handler::Dummy;
#[cfg(feature = "daemon")]
use parity_tokio_ipc::Connection as IPCConnection;
#[cfg(feature = "daemon")]
pub use std::path::Path;
#[cfg(feature = "daemon")]
pub use tap::Pipe;
#[cfg(feature = "daemon")]
use tokio::io::WriteHalf;
#[cfg(feature = "daemon")]
pub type Connection = Compat<WriteHalf<IPCConnection>>;
#[cfg(feature = "daemon")]
pub type NvimConnection = nvim_rs::Neovim<Connection>;
#[cfg(feature = "daemon")]
pub use super::buffer::Buffers;
#[cfg(feature = "daemon")]
pub use nvim_rs::{Buffer, Window};
#[cfg(feature = "daemon")]
pub use tokio_stream::{Stream, StreamExt};
