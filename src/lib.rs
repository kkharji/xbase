//! Daemon Module: Manage projects and Handle requests from neovim
pub mod daemon;

// Submodules
#[cfg(feature = "xcode")]
pub mod xcode;

#[cfg(feature = "compilation")]
pub mod compile;

#[cfg(feature = "xcodegen")]
pub mod xcodegen;

// Utilities
pub mod util;

#[cfg(feature = "server")]
pub mod server;
