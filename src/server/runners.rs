use super::*;
use crate::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tap::Pipe;
use xcodeproj::pbxproj::PBXTargetPlatform;

/// Represntaiton of Project runners index by Platfrom
#[derive(Serialize, Deserialize, TypeScriptify)]
pub struct Runners(pub HashMap<String, Vec<DeviceLookup>>);

pub async fn handle() -> Result<Runners> {
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
                .map(|(id, d)| DeviceLookup {
                    name: d.name.clone(),
                    id: id.clone(),
                })
                .collect::<Vec<_>>(),
        )
    })
    .collect::<HashMap<String, _>>()
    .pipe(Runners)
    .pipe(Ok)
}
