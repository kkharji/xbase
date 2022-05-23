mod build;
mod drop;
mod register;
mod rename_file;
mod run;
mod watch_target;

pub use build::*;
pub use drop::*;
pub use register::*;
pub use rename_file::*;
pub use run::*;
pub use watch_target::*;

#[cfg(feature = "mlua")]
use crate::util::mlua::LuaExtension;

#[cfg(feature = "mlua")]
use mlua::prelude::*;

#[cfg(feature = "mlua")]
use super::Requester;

#[cfg(feature = "daemon")]
use crate::daemon::Handler;

#[cfg(feature = "daemon")]
use crate::Result;

#[cfg(feature = "daemon")]
use async_trait::async_trait;

use crate::client::Client;

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
convertable!(RunRequest);
convertable!(Register);
convertable!(RenameFile);
convertable!(Drop);
convertable!(WatchTarget);
