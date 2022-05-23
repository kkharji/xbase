#[cfg(feature = "daemon")]
use crate::{
    client::Client,
    error::{EnsureOptional, LoopError},
    Result,
};

use crate::types::{Project, Root};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

#[cfg(feature = "daemon")]
use std::path::PathBuf;

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct ProjectStore(pub HashMap<Root, Project>);

impl Deref for ProjectStore {
    type Target = HashMap<Root, Project>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ProjectStore {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// TODO(projects): pressist a list of projects paths and information
#[cfg(feature = "daemon")]
impl ProjectStore {
    pub async fn add(&mut self, client: &Client) -> Result<()> {
        let Client { root, pid, .. } = client;

        let mut project = Project::new(root).await?;

        project.root = root.clone();
        project.clients.push(*pid);

        tracing::info!("AddProject({})", client.abbrev_root());

        self.0.insert(root.to_path_buf(), project);

        Ok(())
    }

    pub fn get_mut(&mut self, root: &PathBuf) -> Result<&mut Project> {
        self.0
            .get_mut(root)
            .ok_or_else(|| LoopError::NoProject(root.into()).into())
    }

    pub fn get(&mut self, root: &PathBuf) -> Result<&Project> {
        self.0
            .get(root)
            .ok_or_else(|| LoopError::NoProject(root.into()).into())
    }

    /// Remove project using root and pid.
    ///
    /// if pid doesn't exists in Project.clients the remove aborts,
    /// if pid exists and it's the only one it will be removed.
    /// if pid removed and there is other pids exists, project will not be removed.
    pub async fn remove(&mut self, client: &Client) -> Result<Option<Project>> {
        let Client { root, pid, .. } = client;

        // Get project with root
        let project = self.0.get_mut(root).to_result("project", root)?;

        // Remove client pid from project.
        project.clients.retain(|p| p != pid);

        // Remove project only when no more client using that data.
        if project.clients.is_empty() {
            tracing::info!("RemoveProject({})", client.abbrev_root());
            return Ok(self.0.remove(root));
        }

        Ok(None)
    }
}
