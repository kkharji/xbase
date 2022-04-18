use crate::state::SharedState;
use crate::DaemonCommandExt;
use anyhow::Result;
use async_trait::async_trait;

#[derive(Debug)]
pub struct Build {
    pub target: Option<String>,
    pub configuration: Option<String>,
    pub scheme: Option<String>,
}

impl Build {
    pub fn new(_args: Vec<&str>) -> Result<Build> {
        Ok(Self {
            target: None,
            configuration: None,
            scheme: None,
        })
    }
}

#[async_trait]
impl DaemonCommandExt for Build {
    async fn handle(&self, _state: SharedState) -> Result<()> {
        tracing::info!("Reanmed command");
        Ok(())
    }
}
