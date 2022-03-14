use crate::project::{Project, Target, TargetMap};
use anyhow::{bail, Ok, Result};
use std::path::{Path, PathBuf};

use ::tracing::{debug, info, trace};
use libproc::libproc::proc_pid;
use notify::EventKind;
use std::process::Stdio;
use tokio::process::Command;

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

        info!("[New::{}]: {:?}", project.name(), root);

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
        let name = self.project.name();
        self.clients.retain(|&pid| {
            if proc_pid::name(pid).is_err() {
                debug!("[Update::{}]: Remove Client: {pid}", name);
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
        debug!("[Update::{}] Add Client: {pid}", self.name());
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

    /// Checks whether current workspace is xcodegen project.
    pub fn is_xcodegen_project(&self) -> bool {
        /*
            TODO: support otherways to identify xcodegen project
            Some would have xcodegen config as json file or
            have different location to where they store xcodegen project config.
        */
        self.root.join("project.yml").exists()
    }

    /// Regenerate compiled commands and xcodeGen if project.yml exists
    pub async fn on_dirctory_change(&self, _path: String, _event: EventKind) -> Result<()> {
        if self.is_xcodegen_project() {
            /*
                FIXME: make xCodeGen binary path configurable.

                Current implementation will not work unless the user has xcodeGen located in
                `~/.mint/bin/xcodegen`. Should either make it configurable as well as support a
                number of paths by default.
            */
            let xcodegen_path = dirs::home_dir().unwrap().join(".mint/bin/xcodegen");
            let xcodegen = Command::new(xcodegen_path)
                .current_dir(self.root.clone())
                .stdout(Stdio::null())
                .arg("generate")
                .spawn()
                .expect("Failed to start xcodeGen.")
                .wait()
                .await
                .expect("Failed to run xcodeGen.");

            if xcodegen.success() {
                info!("Generated {}.xcodeproj", self.name());
            }
        }

        Ok(())
    }
}
