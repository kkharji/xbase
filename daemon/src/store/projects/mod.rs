use crate::broadcast::Broadcast;
use crate::project::{project, Project};
use crate::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use xbase_proto::IntoResult;

#[derive(Default, Debug, derive_deref_rs::Deref)]
pub struct ProjectStore(HashMap<PathBuf, Box<dyn Project + Send>>);

// TODO(projects): presist a list of projects paths and information
impl ProjectStore {
    pub async fn register(&mut self, root: &PathBuf, logger: &Arc<Broadcast>) -> Result<()> {
        let project = project(&root, logger).await?;

        log::info!("[{}] added", project.name());

        self.0.insert(root.into(), project);

        Ok(())
    }

    pub fn get_mut(&mut self, root: &PathBuf) -> Result<&mut Box<dyn Project + Send>> {
        let project = self.0.get_mut(root).into_result("Project".into(), root)?;
        log::trace!("[{}] accessed", project.name());
        Ok(project)
    }

    pub fn get(&self, root: &PathBuf) -> Result<&Box<dyn Project + Send>> {
        let project = self.0.get(root).into_result("Project", root)?;
        log::trace!("[{}] accessed", project.name());
        Ok(project)
    }

    /// Remove project using root and pid.
    ///
    /// if pid doesn't exists in Project.clients the remove aborts,
    /// if pid exists and it's the only one it will be removed.
    /// if pid removed and there is other pids exists, project will not be removed.
    pub async fn remove(&mut self, root: &PathBuf) -> Result<Option<Box<dyn Project + Send>>> {
        // Get project with root
        let project = self.get_mut(root)?;

        // Remove client pid from project.
        project.dec_clients();

        // Remove project only when no more client using that data.
        if project.clients() == &0 {
            log::info!("[{}] removed", project.name());
            return Ok(self.0.remove(root));
        }

        Ok(None)
    }
}
