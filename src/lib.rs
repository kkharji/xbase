#[cfg(feature = "compilation")]
pub mod compile;

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

pub mod constants;
pub mod state;
pub mod store;

#[cfg(feature = "daemon")]
mod error;

#[cfg(feature = "daemon")]
mod run;
#[cfg(feature = "daemon")]
mod watch;

#[cfg(feature = "daemon")]
pub use error::{CompileError, Error, LoopError};

#[cfg(feature = "daemon")]
pub type Result<T> = std::result::Result<T, Error>;
