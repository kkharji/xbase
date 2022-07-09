use super::*;
use serde::{Deserialize, Serialize};
use tap::Pipe;

/// All the requests that xbase can handle
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

impl Request {
    pub async fn handle(self) -> Response {
        match self {
            Request::Register(req) => req.handle().await.pipe(Response::new),
            Request::Build(req) => req.handle().await.pipe(Response::new),
            Request::Run(req) => req.handle().await.pipe(Response::new),
            Request::Drop(req) => req.handle().await.pipe(Response::new),
            Request::GetRunners(req) => req.handle().await.pipe(Response::new),
            Request::GetProjectInfo(req) => req.handle().await.pipe(Response::new),
        }
    }
}
