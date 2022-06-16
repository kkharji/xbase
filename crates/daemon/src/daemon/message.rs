use super::*;
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

/// Daemon Message
#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Build(BuildRequest),
    Run(RunRequest),
    Register(RegisterRequest),
    Drop(DropRequest),
}

impl Message {
    pub async fn handle(self) -> crate::Result<()> {
        match self {
            Self::Build(c) => Handler::handle(c).await,
            Self::Run(c) => Handler::handle(c).await,
            Self::Register(c) => Handler::handle(c).await,
            Self::Drop(c) => Handler::handle(c).await,
        }
    }
}

/// Requirement for daemon handling request
#[async_trait::async_trait]
pub trait Handler: std::fmt::Debug + Sized {
    async fn handle(self) -> crate::Result<()> {
        tracing::error!("Not Implemented! {:#?}", self);
        Ok(())
    }
}
