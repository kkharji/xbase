pub mod build;
pub mod compile;
pub mod constants;
pub mod device;
pub mod drop;
pub mod error;
pub mod nvim;
pub mod project;
pub mod register;
pub mod run;
pub mod state;
pub mod store;
pub mod util;
pub mod watch;
pub mod xcodegen;
pub use error::{CompileError, Error, LoopError};
pub type Result<T> = std::result::Result<T, Error>;

#[async_trait::async_trait]
pub trait RequestHandler {
    async fn handle(self) -> Result<()>
    where
        Self: Sized + std::fmt::Debug,
    {
        tracing::error!("Not Implemented! {:#?}", self);
        Ok(())
    }
}
