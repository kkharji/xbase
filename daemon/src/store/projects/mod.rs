use crate::error::{EnsureOptional, LoopError};
use crate::project::{project, Project};
use crate::Result;
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use xbase_proto::Client;

#[derive(Default, Debug, derive_deref_rs::Deref, Serialize)]
pub struct ProjectStore(HashMap<PathBuf, Box<dyn Project + Send>>);

// TODO(projects): pressist a list of projects paths and information
impl ProjectStore {
    pub async fn add(&mut self, client: &Client) -> Result<()> {
        log::info!("Add: {:?}", client.abbrev_root());

        let key = client.root.to_path_buf();
        let project = project(client).await?;

        self.0.insert(key, project);

        Ok(())
    }

    pub fn get_mut(&mut self, root: &PathBuf) -> Result<&mut Box<dyn Project + Send>> {
        self.0
            .get_mut(root)
            .ok_or_else(|| LoopError::NoProject(root.into()).into())
    }

    pub fn get(&self, root: &PathBuf) -> Result<&Box<dyn Project + Send>> {
        self.0
            .get(root)
            .ok_or_else(|| LoopError::NoProject(root.into()).into())
    }

    /// Remove project using root and pid.
    ///
    /// if pid doesn't exists in Project.clients the remove aborts,
    /// if pid exists and it's the only one it will be removed.
    /// if pid removed and there is other pids exists, project will not be removed.
    pub async fn remove(&mut self, client: &Client) -> Result<Option<Box<dyn Project + Send>>> {
        let Client { root, pid, .. } = client;

        // Get project with root
        let project = self.0.get_mut(root).to_result("project", root)?;

        // Remove client pid from project.
        project.clients_mut().retain(|p| p != pid);

        // Remove project only when no more client using that data.
        if project.clients().is_empty() {
            log::info!("Remove: {:?}", client.abbrev_root());
            return Ok(self.0.remove(root));
        }

        Ok(None)
    }
}
