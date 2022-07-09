mod methods;
mod stream;

use crate::{error::Error, types::Result};
use methods::*;
use serde::{Deserialize, Serialize};
use tap::Pipe;
use tokio::net::UnixStream;

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
    /// Get available runners
    GetRunners(GetRunnersRequest),
    /// Get project info that might change between calls, like targets or watchlist
    GetProjectInfo(GetProjectInfoRequest),
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
        fn as_response<T: Serialize>(v: Result<T>) -> Response {
            let mut response = Response::default();
            match v {
                Ok(data) => response.data = serde_json::to_value(data).unwrap().into(),
                Err(error) => response.error = error.into(),
            };
            response
        }
        match self {
            Request::Register(req) => req.handle().await.pipe(as_response),
            Request::Build(req) => req.handle().await.pipe(as_response),
            Request::Run(req) => req.handle().await.pipe(as_response),
            Request::Drop(req) => req.handle().await.pipe(as_response),
            Request::GetRunners(req) => req.handle().await.pipe(as_response),
            Request::GetProjectInfo(req) => req.handle().await.pipe(as_response),
        }
    }
}

/// Future that await and process client requests.
pub async fn handle(mut stream: UnixStream) {
    use futures::{FutureExt, SinkExt, TryStreamExt};

    tracing::info!("Handling a new client");
    let (mut reader, mut writer) = stream::split(&mut stream);
    loop {
        match reader.try_next().await {
            Ok(Some(request)) => {
                request
                    .handle()
                    .then(|res| writer.send(res))
                    .await
                    .map_err(|err| tracing::error!("Fail to send response: {err}"))
                    .ok();
            }
            Err(err) => tracing::error!("Fail to read request: {err:#?}"),
            Ok(None) => break,
        }
    }
    tracing::info!("Disconnecting a client");
}
