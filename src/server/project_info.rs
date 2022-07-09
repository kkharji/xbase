use super::*;
use crate::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Request to Get `ProjectInfo`
#[derive(Debug, Serialize, Deserialize, TypeScriptify)]
pub struct GetProjectInfoRequest {
    root: PathBuf,
}

#[derive(Debug, Serialize, TypeScriptify)]
pub struct ProjectInfo {
    /// Get watched configurations for given root
    watchlist: Vec<String>,
    /// Get targets information for a registers project with a given root
    targets: HashMap<String, TargetInfo>,
}

#[async_trait]
impl RequestHandler<ProjectInfo> for GetProjectInfoRequest {
    async fn handle(self) -> Result<ProjectInfo> {
        let listeners = &self.root.try_get_watcher().await?.listeners;
        let project = self.root.try_get_project().await?;

        Ok(ProjectInfo {
            watchlist: listeners.iter().map(|(k, _)| k.clone()).collect(),
            targets: project.targets().clone(),
        })
    }
}
