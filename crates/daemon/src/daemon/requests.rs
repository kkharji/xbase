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

use crate::client::Client;
use crate::daemon::Handler;
use crate::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

macro_rules! convertable {
    ($type:ident) => {
        paste::paste! {
            impl From<[<$type Request>]> for super::Request {
                fn from(msg: [<$type Request>]) -> Self {
                    let message = super::Message::$type(msg);
                    Self { message }
                }
            }
        }
    };
}

convertable!(Build);
convertable!(Run);
convertable!(Register);
convertable!(Drop);

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, strum::Display, strum::EnumString)]

pub enum RequestOps {
    Watch,
    Stop,
    Once,
}

impl Default for RequestOps {
    fn default() -> Self {
        Self::Once
    }
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
