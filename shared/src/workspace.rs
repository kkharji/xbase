use crate::project::{Project, Target, TargetMap};
use anyhow::{bail, Ok, Result};
use std::path::PathBuf;

use crate::tracing;
use libproc::libproc::proc_pid;

/// Managed Workspace
#[derive(Debug)]
pub struct Workspace {
    /// Project root path
    pub root: PathBuf,
    /// Project.yml base content
    pub project: Project,
    /// Active clients pids connect for the xcodebase workspace.
    pub clients: Vec<i32>,
}

impl Workspace {
    /// Create new workspace from a path representing project root.
    /// TODO: Support setting up projects with .xproj as well as xcworkspace
    pub async fn new(root: &str) -> Result<Self> {
        let root = PathBuf::from(root);

        let project = {
            let path = root.join("project.yml");
            if !path.exists() {
                bail!("project.yaml doesn't exist in '{:?}'", root)
            }

            Project::new_from_project_yml(path).await?
        };

        Ok(Self {
            root,
            project,
            clients: vec![],
        })
    }

    pub async fn new_with_client(root: &str, pid: i32) -> Result<Self> {
        let mut ws = Workspace::new(&root).await?;
        ws.add_client(pid);
        Ok(ws)
    }

    pub fn update_clients(&mut self) {
        self.clients.retain(|&pid| {
            if proc_pid::name(pid).is_err() {
                tracing::trace!("Removeing {pid}");
                false
            } else {
                true
            }
        });
    }

    /// Add new client to workspace (implicitly check if all other clients are stil valid).
    pub fn add_client(&mut self, pid: i32) {
        // Remove no longer active clients
        self.update_clients();
        // NOTE: Implicitly assuming that pid is indeed a valid pid
        self.clients.push(pid)
    }

    /// Wrapper around Project.name:
    /// Returns project name
    pub fn name(&self) -> &str {
        self.project.name()
    }

    /// Wrapper around project.targets
    /// Returns all avaliable targets
    pub fn targets(&self) -> &TargetMap {
        self.project.targets()
    }

    /// Get project target from project.targets using target_name
    pub fn get_target(&self, target_name: &str) -> Option<&Target> {
        self.project.targets().get(target_name)
    }
}
