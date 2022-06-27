pub mod build;
pub mod compile;
pub mod constants;
pub mod device;
pub mod drop;
pub mod logger;
pub mod nvim;
pub mod project;
pub mod register;
pub mod run;
pub mod state;
pub mod store;
pub mod util;
pub mod watch;

use process_stream::Stream;
use std::pin::Pin;

pub use xbase_proto::{Error, IntoResult, Result};
pub type StringStream = Pin<Box<dyn Stream<Item = String> + Send>>;
