#![feature(result_flattening)]
#[cfg(feature = "compilation")]
pub mod compile;

#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "xcodegen")]
pub mod xcodegen;

#[cfg(any(feature = "daemon", feature = "mlua"))]
pub mod daemon;

#[cfg(feature = "serial")]
pub mod types;

pub mod util;

#[cfg(any(feature = "daemon", feature = "lua"))]
mod client;
#[cfg(any(feature = "daemon", feature = "lua"))]
mod nvim;

#[cfg(feature = "daemon")]
mod xcode;

#[cfg(feature = "daemon")]
pub mod watcher;

pub mod state;

pub mod constants;

pub mod store;

#[cfg(any(feature = "daemon", feature = "server"))]
mod error;

#[cfg(feature = "daemon")]
mod runner;

#[cfg(any(feature = "daemon", feature = "server"))]
pub use error::{CompileError, Error, LoopError, WatchError};

#[cfg(any(feature = "daemon", feature = "server"))]
pub type Result<T> = std::result::Result<T, Error>;
