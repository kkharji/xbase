use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use std::{path::PathBuf, sync::Arc};
use {super::*, crate::*};

/// Request to build a particular project
#[derive(Debug, Serialize, Deserialize, TypeDef)]
pub struct BuildRequest {
    pub root: PathBuf,
    pub settings: BuildSettings,
    pub operation: Operation,
}

#[async_trait]
impl RequestHandler<()> for BuildRequest {
    async fn handle(self) -> Result<()> {
        tracing::trace!("{:#?}", self);
        runtimes()
            .await
            .get(&self.root)
            .ok_or_else(|| Error::UnknownProject(self.root.clone()))
            .map(|r| r.send(PRMessage::Build(self)))
    }
}

impl Display for BuildRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:Build:{}", self.root.display(), self.settings)
    }
}

#[async_trait]
impl Watchable for BuildRequest {
    async fn trigger(&self, p: &mut ProjectImpl, _: &Event, b: &Arc<Broadcast>) -> Result<()> {
        p.build(&self.settings, None, b)?;
        Ok(())
    }

    /// A function that controls whether a a Watchable should restart
    async fn should_trigger(&self, event: &Event) -> bool {
        event.is_any_but_not_seen()
    }

    /// A function that controls whether a watchable should be droped
    async fn should_discard(&self, _event: &Event) -> bool {
        false
    }

    /// Drop watchable for watching a given file system
    async fn discard(&self) {}
}
