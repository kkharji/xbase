use crate::state::SharedState;
use crate::{Daemon, DaemonCommandExt};
use anyhow::{bail, Result};
use async_trait::async_trait;

/// Rename file + class
#[derive(Debug)]
pub struct RenameFile {
    pub path: String,
    pub name: String,
    pub new_name: String,
}

impl RenameFile {
    pub fn new(args: Vec<&str>) -> Result<Self> {
        let path = args.get(0);
        let new_name = args.get(2);
        let name = args.get(1);
        if path.is_none() || name.is_none() || new_name.is_none() {
            bail!(
                "Missing arugments: [path: {:?}, old_name: {:?}, name_name: {:?} ]",
                path,
                name,
                new_name
            )
        }

        Ok(Self {
            path: path.unwrap().to_string(),
            name: name.unwrap().to_string(),
            new_name: new_name.unwrap().to_string(),
        })
    }

    pub fn request(path: &str, name: &str, new_name: &str) -> Result<()> {
        Daemon::execute(&["rename_file", path, name, new_name])
    }
}

#[async_trait]
impl DaemonCommandExt for RenameFile {
    async fn handle(&self, _state: SharedState) -> Result<()> {
        tracing::info!("Reanmed command");
        Ok(())
    }
}
