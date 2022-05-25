mod build;
mod drop;
mod register;
mod rename_file;
mod run;

pub use build::*;
pub use drop::*;
pub use register::*;
pub use rename_file::*;
pub use run::*;

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
convertable!(BuildRequest);
convertable!(RunRequest);
convertable!(Register);
convertable!(RenameFile);
convertable!(Drop);

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, strum::Display, strum::EnumString,
)]

pub enum RequestOps {
    Watch,
    Stop,
    #[default]
    Once,
}

impl RequestOps {
    /// Returns `true` if the request kind is [`Watch`].
    ///
    /// [`Watch`]: RequestKind::Watch
    #[must_use]
    pub fn is_watch(&self) -> bool {
        matches!(self, Self::Watch)
    }

    /// Returns `true` if the request kind is [`WatchStop`].
    ///
    /// [`WatchStop`]: RequestKind::WatchStop
    #[must_use]
    pub fn is_stop(&self) -> bool {
        matches!(self, Self::Stop)
    }

    /// Returns `true` if the request kind is [`Once`].
    ///
    /// [`Once`]: RequestKind::Once
    #[must_use]
    pub fn is_once(&self) -> bool {
        matches!(self, Self::Once)
    }
}

#[cfg(feature = "lua")]
impl<'a> FromLua<'a> for RequestOps {
    fn from_lua(lua_value: LuaValue<'a>, _lua: &'a Lua) -> LuaResult<Self> {
        use std::str::FromStr;
        if let LuaValue::String(value) = lua_value {
            let value = value.to_string_lossy();
            Self::from_str(&*value).to_lua_err()
        } else {
            Ok(Self::default())
        }
    }
}
