use super::*;

/// Rename file + class
#[derive(Debug, Serialize, Deserialize)]
pub struct RenameFile {
    pub client: Client,
}

// TODO: Implement file rename along with it's main class if any.
#[async_trait]
impl Handler for RenameFile {
    async fn handle(self) -> Result<()> {
        tracing::info!("Reanmed command");
        Ok(())
    }
}
