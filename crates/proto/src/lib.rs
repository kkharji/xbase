mod message;
mod types;
mod util;
pub use message::*;
pub use types::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub message: Message,
}

impl Request {
    pub fn read(value: String) -> Result<Self, serde_json::Error> {
        serde_json::from_str(value.trim())
    }
}

#[async_trait::async_trait]
pub trait RequestHandler {
    async fn handle(self) -> Result<(), Box<dyn std::error::Error>>
    where
        Self: Sized + std::fmt::Debug,
    {
        tracing::error!("Not Implemented! {:#?}", self);
        Ok(())
    }
}
