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

pub use error::{CompileError, Error, LoopError};
use process_stream::{ProcessItem, Stream};
use std::pin::Pin;

pub type Result<T> = std::result::Result<T, Error>;
pub type OutputStream = Pin<Box<dyn Stream<Item = ProcessItem> + Send>>;
pub type StringStream = Pin<Box<dyn Stream<Item = String> + Send>>;

#[async_trait::async_trait]
pub trait RequestHandler {
    async fn handle(self) -> Result<()>
    where
        Self: Sized + std::fmt::Debug,
    {
        log::error!("Not Implemented! {:#?}", self);
        Ok(())
    }
}
