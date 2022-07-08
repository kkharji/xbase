use crate::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetTargetsRequest {
    root: PathBuf,
}

#[async_trait]
impl RequestHandler<HashMap<String, TargetInfo>> for GetTargetsRequest {
    async fn handle(self) -> Result<HashMap<String, TargetInfo>> {
        self.root
            .try_get_project()
            .await
            .map(|p| p.targets().clone())
    }
}
