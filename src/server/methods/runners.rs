use crate::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetRunnersRequest;

#[async_trait]
impl RequestHandler<HashMap<String, Vec<HashMap<String, String>>>> for GetRunnersRequest {
    async fn handle(self) -> Result<HashMap<String, Vec<HashMap<String, String>>>> {
        let devices = devices();
        vec![
            PBXTargetPlatform::IOS,
            PBXTargetPlatform::WatchOS,
            PBXTargetPlatform::TvOS,
        ]
        .into_iter()
        .map(|p| {
            (
                p.to_string(),
                devices
                    .iter()
                    .filter(|(_, d)| d.platform == p)
                    .map(|(id, d)| {
                        HashMap::from([("id".into(), id.into()), ("name".into(), d.name.clone())])
                    })
                    .collect::<Vec<HashMap<_, _>>>(),
            )
        })
        .collect::<HashMap<String, _>>()
        .pipe(Ok)
    }
}
