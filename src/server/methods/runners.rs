use crate::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetRunnersRequest {
    platform: PBXTargetPlatform,
}

#[async_trait]
impl RequestHandler<Vec<HashMap<String, String>>> for GetRunnersRequest {
    async fn handle(self) -> Result<Vec<HashMap<String, String>>> {
        devices()
            .iter()
            .filter(|(_, d)| d.platform == self.platform)
            .map(|(id, d)| {
                HashMap::from([("id".into(), id.clone()), ("name".into(), d.name.clone())])
            })
            .collect::<Vec<HashMap<String, String>>>()
            .pipe(Ok)
    }
}
