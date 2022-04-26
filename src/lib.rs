mod daemon;
pub use daemon::*;

mod state;
pub use state::*;

// Submodules

#[cfg(feature = "xcode")]
pub mod xcode;

#[cfg(feature = "compilation")]
pub mod compile;

#[cfg(feature = "xcodegen")]
pub mod xcodegen;

// Utilities

pub mod util;

#[cfg(feature = "lua")]
pub use util::mlua::LuaExtension;

#[cfg(feature = "logging")]
pub use util::tracing::install_tracing;

#[cfg(feature = "watcher")]
pub use util::watch;

#[cfg(feature = "server")]
pub mod server;
