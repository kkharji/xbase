use crate::{
    daemon::{WatchStart, Workspace},
    nvim::Client,
    watchers::{
        event_handler, new_watch_handler, recompile_handler, ProjectWatchers, TargetWatchers,
    },
};
use anyhow::Result;
use std::collections::HashMap;
use tap::Pipe;
use tracing::{error, info};

use super::set_watch_script;

/// Main state
#[derive(Default, Debug)]
pub struct DaemonStateData {
    /// Managed workspaces
    pub workspaces: HashMap<String, Workspace>,
    /// Connected clients process id
    pub clients: Vec<i32>,
    /// project recompile watcher
    pub build_watchers: ProjectWatchers,
    /// target recompile watcher
    pub target_watchers: TargetWatchers,
}

pub type DaemonState = std::sync::Arc<tokio::sync::Mutex<DaemonStateData>>;

impl DaemonStateData {
    pub fn get_workspace(&self, root: &str) -> Result<&Workspace> {
        match self.workspaces.get(root) {
            Some(o) => Ok(o),
            None => anyhow::bail!("No workspace for {}", root),
        }
    }

    pub fn get_mut_workspace(&mut self, root: &str) -> Result<&mut Workspace> {
        match self.workspaces.get_mut(root) {
            Some(o) => Ok(o),
            None => anyhow::bail!("No workspace for {}", root),
        }
    }

    pub async fn add_workspace(
        &mut self,
        root: &str,
        pid: i32,
        address: &str,
        state: DaemonState,
    ) -> Result<()> {
        // TODO: Support projects with .xproj as well as xcworkspace

        if self.workspaces.contains_key(root) {
            let ws = self.get_mut_workspace(root).unwrap();
            return ws.add_nvim_client(pid, address).await;
        }

        let workspace = Workspace::new(root, pid, address).await?;
        let root = root.to_string();

        self.watch(&root, None, state).await?;
        self.workspaces.insert(root, workspace);

        tracing::trace!("{:#?}", self);

        Ok(())
    }

    // Remove remove client from workspace and the workspace if it's this client was the last one.
    pub async fn remove_workspace(&mut self, root: &str, pid: i32) -> Result<()> {
        let mut name = None;

        if let Some(ws) = self.workspaces.get_mut(root) {
            let clients_len = ws.remove_client(pid);
            clients_len
                .eq(&0)
                .then(|| name = ws.project.name().to_string().into());
        } else {
            error!("'{root}' is not a registered workspace!");
        }

        if let Some(name) = name {
            info!("Dropping [{}] {:?}", name, root);

            if self.build_watchers.contains_key(root) {
                self.stop_watch(root, None).await;
            }

            self.workspaces.remove(root);
        }

        Ok(())
    }

    /// Stop a watch service
    async fn stop_watch(&mut self, root: &str, client: Option<Client>) -> Result<()> {
        if client.is_some() {
            self.validate(client).await?;
        } else {
            if let Some(handle) = self.build_watchers.get(root) {
                handle.abort();
                tracing::debug!("Project Watch service stopeed");
                self.build_watchers.remove(root);
            }
        }
        Ok(())
    }

    pub async fn validate(&mut self, client: Option<Client>) -> Result<()> {
        if let Some(client) = client {
            let mut dropped_targets = vec![];
            let root = client.root.clone();

            self.target_watchers.retain(|target, (req, handle)| {
                if req.client.root == client.root && req.client.pid == client.pid {
                    handle.abort();
                    // if let Err(err) = handle.await {
                    //     if !err.is_cancelled() {
                    //         error!("handler is not cancelled for {:#?}", self);
                    //     }
                    // }
                    dropped_targets.push(target.clone());
                    true
                } else {
                    false
                }
            });

            let ws = self.get_workspace(&root)?;
            let scripts = dropped_targets
                .iter()
                .map(|target| set_watch_script(&client.root, target, false))
                .collect::<Vec<String>>();

            for (_, nvim) in ws.clients.iter() {
                for script in scripts.iter() {
                    nvim.exec_lua(script, vec![]).await?;
                }
            }
        } else {
            use crate::util::proc_exists;
            self.clients
                .retain(|pid| proc_exists(pid, || info!("Removing {pid}")));
            self.workspaces
                .iter_mut()
                .for_each(|(_, ws)| ws.ensure_active_clients())
        }
        Ok(())
    }

    pub async fn watch(
        &mut self,
        root: &str,
        watch_req: Option<WatchStart>,
        state: DaemonState,
    ) -> Result<()> {
        if let Some(watch_req) = watch_req {
            let ref mut watchers = self.target_watchers;
            let target = watch_req.request.target.clone();

            tracing::info!("Watch target [{}]", watch_req.request.target);
            new_watch_handler(root.into(), state, event_handler, target.clone().into())
                .pipe(|handler| watchers.insert(target.clone(), (watch_req, handler)));

            let ws = self.get_mut_workspace(&root)?;
            let update_script = set_watch_script(&root, &target, true);

            for (_, nvim) in ws.clients.iter() {
                nvim.exec_lua(&update_script, vec![]).await?;
            }
        } else {
            let ref mut watchers = self.build_watchers;
            new_watch_handler(root.into(), state, recompile_handler, None)
                .pipe(|handler| watchers.insert(root.into(), handler));
        }

        Ok(())
    }
}
