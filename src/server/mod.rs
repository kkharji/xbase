mod methods;
pub mod stream;

use crate::{error::Error, types::Result};
use methods::*;
use serde::{Deserialize, Serialize};
use tap::Pipe;

/// Trait that must be implemented by All Request members
#[async_trait::async_trait]
pub trait RequestHandler<T: Serialize> {
    async fn handle(self) -> Result<T>;
}

/// Server Requests
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method", rename_all = "snake_case")]
pub enum Request {
    /// Register project root and get broadcaster reader file description
    Register(RegisterRequest),
    /// Build Project and get path to where to build log will be located
    Build(BuildRequest),
    /// Run Project and get path to where to Runtime log will be located
    Run(RunRequest),
    /// Drop projects at a given roots
    Drop(DropRequest),
    /// Get targets information for a registers project with a given root
    GetTargets(GetTargetsRequest),
    /// Get available runners
    GetRunners(GetRunnersRequest),
}

/// Server Response
#[derive(Default, Debug, Serialize)]
pub struct Response {
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Error>,
}

impl Request {
    pub async fn handle(self) -> Response {
        let response = match self {
            // TODO: return a raw_fd
            Request::Register(req) => req.handle().await.pipe(Response::new),
            Request::Build(req) => req.handle().await.pipe(Response::new),
            Request::Run(req) => req.handle().await.pipe(Response::new),
            Request::Drop(req) => req.handle().await.pipe(Response::new),
            Request::GetTargets(req) => req.handle().await.pipe(Response::new),
            Request::GetRunners(req) => req.handle().await.pipe(Response::new),
        };
        tracing::info!("{response:#?}");
        response
    }
}

impl Response {
    pub fn new<T: Serialize>(v: Result<T>) -> Self {
        let mut response = Self::default();
        match v {
            Ok(data) => response.data = serde_json::to_value(data).unwrap().into(),
            Err(error) => response.error = error.into(),
        };
        response
    }
}
