mod build;
mod drop;
mod project_info;
mod register;
mod rename_file;
mod run;
mod watch_target;

pub use build::Build;
pub use drop::Drop;
pub use project_info::ProjectInfo;
pub use register::Register;
pub use rename_file::RenameFile;
pub use run::Run;
pub use watch_target::WatchTarget;

#[cfg(feature = "mlua")]
use crate::util::mlua::LuaExtension;

#[cfg(feature = "mlua")]
use mlua::prelude::*;

#[cfg(feature = "mlua")]
use super::Requester;

#[cfg(feature = "daemon")]
use crate::daemon::Handler;

#[cfg(feature = "daemon")]
use anyhow::Result;

#[cfg(feature = "daemon")]
use async_trait::async_trait;

use crate::types::Client;

use serde::{Deserialize, Serialize};

macro_rules! convertable {
    ($type:ident) => {
        impl From<$type> for super::Request {
            fn from(msg: $type) -> Self {
                let message = super::Message::$type(msg);
                Self { message }
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
convertable!(WatchTarget);
