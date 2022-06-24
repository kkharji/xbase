mod error;
mod message;
mod types;
mod util;
use std::path::PathBuf;

pub use error::*;
pub use message::*;
pub use types::*;

use serde::{Deserialize, Serialize};

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub message: Message,
}

impl Request {
    pub fn read(value: String) -> Result<Self> {
        Ok(serde_json::from_str(value.trim())?)
    }
}

#[tarpc::service]
pub trait XBase {
    /// Register project root with a path to setup logs
    async fn register(req: RegisterRequest) -> Result<PathBuf>;
    /// Build Project and get path to where to build log will be located
    async fn build(req: BuildRequest) -> Result<PathBuf>;
    /// Run Project and get path to where to Runtime log will be located
    async fn run(req: RunRequest) -> Result<PathBuf>;
    /// Drop project root
    async fn drop(req: DropRequest) -> Result<()>;
}

pub use tarpc::context::{self, Context};
pub use tarpc::serde_transport as transport;
pub use tarpc::server::{BaseChannel, Channel};
pub use tokio_serde::formats::Json;
pub use tokio_util::codec::length_delimited::LengthDelimitedCodec;
