mod error;
mod message;
mod request;
mod types;
mod util;

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

pub use error::*;
pub use message::*;
pub use request::*;
pub use tarpc::context::{self, Context};
pub use tarpc::serde_transport as transport;
pub use tarpc::server::{BaseChannel, Channel};
pub use tokio_serde::formats::Json;
pub use tokio_util::codec::length_delimited::LengthDelimitedCodec;
pub use types::*;
pub use util::PathExt;
use xcodeproj::pbxproj::PBXTargetPlatform;
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Short hand to get Result with given Error
/// based by anyhow's
#[allow(non_snake_case)]
pub fn OK<T>(t: T) -> Result<T> {
    Result::Ok(t)
}

#[tarpc::service]
pub trait XBase {
    ///
    /// Register client and project root.
    ///
    /// If the project is already registered then it will not be re-registered and instead
    /// broadcast address socket will be returned.
    ///
    async fn register(root: PathBuf) -> Result<PathBuf>;
    ///
    /// Build Project and get path to where to build log will be located
    ///
    async fn build(req: BuildRequest) -> Result<()>;
    ///
    /// Run Project and get path to where to Runtime log will be located
    ///
    async fn run(req: RunRequest) -> Result<()>;
    ///
    /// Drop project at a given root
    ///
    async fn drop(roots: HashSet<PathBuf>) -> Result<()>;
    ///
    /// Return targets for client projects
    ///
    async fn targets(root: PathBuf) -> Result<HashMap<String, TargetInfo>>;
    ///
    /// Return device names and id for given target
    ///
    async fn runners(platform: PBXTargetPlatform) -> Result<Vec<HashMap<String, String>>>;
    ///
    /// Return all watched keys
    ///
    async fn watching(root: PathBuf) -> Result<Vec<String>>;
}
