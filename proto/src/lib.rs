mod error;
mod logging;
mod message;
mod types;
mod util;

pub use error::*;
pub use logging::LoggingTask;
pub use message::*;
pub use types::*;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[tarpc::service]
pub trait XBase {
    /// Register project root with vec of logging tasks that might be resulted from registering the
    /// project or getting ongoing logging tasks from different client instance
    async fn register(req: RegisterRequest) -> Result<Vec<LoggingTask>>;
    /// Build Project and get path to where to build log will be located
    async fn build(req: BuildRequest) -> Result<LoggingTask>;
    /// Run Project and get path to where to Runtime log will be located
    async fn run(req: RunRequest) -> Result<LoggingTask>;
    /// Drop project root
    async fn drop(req: DropRequest) -> Result<()>;
}

pub use tarpc::context::{self, Context};
pub use tarpc::serde_transport as transport;
pub use tarpc::server::{BaseChannel, Channel};
pub use tokio_serde::formats::Json;
pub use tokio_util::codec::length_delimited::LengthDelimitedCodec;
