mod build;
mod drop;
mod project_info;
mod register;
mod request;
mod response;
mod run;
mod runners;
pub mod stream;

use typescript_definitions::TypeScriptify;
pub use {
    build::*, drop::*, project_info::*, register::*, request::*, response::*, run::*, runners::*,
};

/// Trait that must be implemented by All Request members
#[async_trait::async_trait]
pub trait RequestHandler<T: serde::Serialize> {
    async fn handle(self) -> crate::Result<T>;
}
