mod build;
mod drop;
mod project_info;
mod register;
mod rename_file;
mod run;
mod watch_start;
mod watch_stop;

pub use build::Build;
pub use drop::Drop;
pub use project_info::ProjectInfo;
pub use register::Register;
pub use rename_file::RenameFile;
pub use run::Run;
pub use watch_start::WatchStart;
pub use watch_stop::WatchStop;

use crate::nvim::Client;

#[cfg(feature = "mlua")]
use crate::util::mlua::LuaExtension;

#[cfg(feature = "mlua")]
use mlua::prelude::*;

#[cfg(feature = "mlua")]
use super::Requester;

#[cfg(feature = "daemon")]
use crate::daemon::Handler;

#[cfg(feature = "daemon")]
use crate::state::DaemonState;

#[cfg(feature = "daemon")]
use anyhow::Result;

#[cfg(feature = "daemon")]
use async_trait::async_trait;

use serde::{Deserialize, Serialize};

macro_rules! convertable {
    ($type:ident) => {
        impl From<$type> for super::Request {
            fn from(msg: $type) -> Self {
                Self {
                    message: super::Message::$type(msg),
                }
            }
        }
    };
}
convertable!(Build);
convertable!(Run);
convertable!(Register);
convertable!(RenameFile);
convertable!(Drop);
convertable!(ProjectInfo);
convertable!(WatchStart);
convertable!(WatchStop);
