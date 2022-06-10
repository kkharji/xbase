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

pub mod constants;
pub mod state;
pub mod store;

#[cfg(any(feature = "daemon", feature = "server"))]
mod error;

#[cfg(feature = "daemon")]
mod run;
#[cfg(feature = "daemon")]
mod watch;

#[cfg(any(feature = "daemon", feature = "server"))]
pub use error::{CompileError, Error, LoopError};

#[cfg(any(feature = "daemon", feature = "server"))]
pub type Result<T> = std::result::Result<T, Error>;
