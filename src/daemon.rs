//! Handle requests from neovim and manage dev workflow
mod message;
mod requests;

pub use message::*;
pub use requests::*;

#[cfg(feature = "daemon")]
pub mod workspace;
#[cfg(feature = "daemon")]
pub use workspace::Workspace;

/// Representation of daemon
pub struct Daemon;
