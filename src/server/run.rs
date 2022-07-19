use super::*;
use crate::runtime::PRMessage;
use crate::{runner::*, *};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use std::path::PathBuf;

/// Request to Run a particular project.
#[derive(Debug, Serialize, Deserialize, TypeDef)]
pub struct RunRequest {
    pub root: PathBuf,
    pub settings: BuildSettings,
    #[serde(default)]
    pub device: Option<DeviceLookup>,
    pub operation: Operation,
}

#[async_trait]
impl RequestHandler<()> for RunRequest {
    async fn handle(self) -> Result<()> {
        tracing::trace!("{:#?}", self);
        runtimes()
            .await
            .get(&self.root)
            .ok_or_else(|| Error::UnknownProject(self.root.clone()))
            .map(|r| r.send(PRMessage::Run(self)))
    }
}

impl Display for RunRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let device = if let Some(device) = self.device.as_ref() {
            device.name.clone()
        } else {
            "Bin".into()
        };
        let settings = &self.settings;
        write!(f, "{}:Run:{device}:{settings}", self.root.display())
    }
}

impl RunRequest {
    pub fn into_service(self) -> RunService {
        let key = self.to_string();
        let Self { settings, root, .. } = self;
        let device = Devices::from_lookup(self.device);

        RunService::new(device, root, settings, key)
    }
}
