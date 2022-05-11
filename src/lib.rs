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

mod nvim;

#[cfg(feature = "daemon")]
mod xcode;

pub mod state;

pub mod constants;
#[cfg(feature = "daemon")]
pub mod watchers;
