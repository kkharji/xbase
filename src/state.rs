#[cfg(feature = "server")]
use crate::compile::{CompilationCommand, CompilationDatabase, CompileFlags};

#[cfg(feature = "server")]
use std::collections::HashMap;

#[cfg(feature = "server")]
use std::path::Path;

#[cfg(any(feature = "server", feature = "daemon"))]
use anyhow::Result;

#[cfg(any(feature = "server", feature = "daemon"))]
use std::path::PathBuf;

#[cfg(any(feature = "server", feature = "daemon"))]
use tap::Pipe;

/// Build Server State.
#[derive(Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct State {
    #[cfg(feature = "server")]
    pub compile_commands: HashMap<PathBuf, CompilationDatabase>,
    #[cfg(feature = "server")]
    pub file_flags: HashMap<PathBuf, CompileFlags>,
    /// Managed Workspaces
    #[cfg(feature = "daemon")]
    pub projects: crate::store::ProjectStore,
    /// Managed Clients
    #[cfg(feature = "daemon")]
    pub clients: crate::store::ClientStore,
    /// Managed watchers
    #[cfg(feature = "daemon")]
    pub watcher: crate::store::WatchStore,
}

#[cfg(feature = "server")]
impl State {
    /// Get [`CompilationDatabase`] for a .compile file path.
    pub fn compile_commands(&mut self, compile_filepath: &Path) -> Result<&CompilationDatabase> {
        if self.compile_commands.contains_key(compile_filepath) {
            self.compile_commands.get(compile_filepath)
        } else {
            crate::compile::parse_from_file(compile_filepath)?
                .pipe(|cmds| self.compile_commands.insert(compile_filepath.into(), cmds))
                .pipe(|_| self.compile_commands.get(compile_filepath))
        }
        .ok_or_else(|| anyhow::anyhow!("No CompilationDatabase found for {:?}", compile_filepath))
    }

    /// Get [`CompileFlags`] for a file
    pub fn file_flags(
        &mut self,
        filepath: &Path,
        compile_filepath: Option<&PathBuf>,
    ) -> Result<&CompileFlags> {
        if let Some(compile_filepath) = compile_filepath {
            if self.file_flags.contains_key(filepath) {
                self.file_flags.get(filepath)
            } else {
                self.compile_commands(compile_filepath)?
                    .iter()
                    .flat_map(CompilationCommand::compile_flags)
                    .flatten()
                    .collect::<HashMap<_, _>>()
                    .pipe(|map| self.file_flags.extend(map))
                    .pipe(|_| self.file_flags.get(filepath))
            }
        } else {
            CompileFlags::from_filepath(filepath)?
                .pipe(|flags| self.file_flags.insert(filepath.to_path_buf(), flags))
                .pipe(|_| self.file_flags.get(filepath))
        }
        .ok_or_else(|| anyhow::anyhow!("Couldn't find file flags for {:?}", filepath))
    }

    /// Clear [`BuildServerState`]
    pub fn clear(&mut self) {
        self.file_flags = Default::default();
        self.compile_commands = Default::default();
    }
}

#[cfg(feature = "daemon")]
impl State {
    /// Get all projects that client have access to it.
    #[allow(dead_code)]
    async fn get_client_projects<'a>(
        &'a self,
        client: &'a crate::types::Client,
    ) -> Result<Vec<(&'a PathBuf, &'a crate::types::Project)>> {
        self.projects
            .iter()
            .filter(|(_, p)| p.clients.contains(&client.pid))
            .collect::<Vec<(&'a PathBuf, &'a crate::types::Project)>>()
            .pipe(Ok)
    }

    #[allow(dead_code)]
    async fn get_clients_with_project_root(
        &self,
        root: &PathBuf,
    ) -> Result<Vec<(&i32, &crate::nvim::NvimClient)>> {
        self.clients
            .iter()
            .filter(|(_, nvim)| nvim.roots.contains(root))
            .collect::<Vec<(&i32, &crate::nvim::NvimClient)>>()
            .pipe(Ok)
    }

    pub fn try_into_string(&self) -> Result<String> {
        Ok(serde_json::to_string(&self)?)
    }

    pub async fn sync_client_state(&self) -> Result<()> {
        let state_str = self.try_into_string()?;
        let update_state_script = format!("vim.g.xcodebase= vim.json.decode([[{state_str}]])");
        tracing::info!("Syncing state to all nvim instance");

        self.clients.update_state(&update_state_script).await?;

        Ok(())
    }

    pub async fn validate(&mut self) {
        let mut invalid_pids = vec![];

        self.clients.retain(|pid, _| {
            crate::util::proc_exists(pid, || {
                tracing::error!("{pid} no longer valid");
                invalid_pids.push(*pid);
            })
        });

        if !invalid_pids.is_empty() {
            for pid in invalid_pids.iter() {
                self.projects.iter_mut().for_each(|(_, p)| {
                    p.clients.retain(|client_pid| pid != client_pid);
                });
            }
            self.projects.retain(|_, p| !p.clients.is_empty())
        }
    }
}
