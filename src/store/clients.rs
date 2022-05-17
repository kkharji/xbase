use crate::daemon::Register;
use crate::nvim::NvimClient;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use tap::Pipe;

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct ClientStore(HashMap<i32, NvimClient>);

impl Deref for ClientStore {
    type Target = HashMap<i32, NvimClient>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ClientStore {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ClientStore {
    pub async fn add(&mut self, req: &Register) -> Result<()> {
        tracing::info!("AddClient({})", req.client.pid);
        NvimClient::new(req)
            .await?
            .pipe(|client| self.insert(req.client.pid, client))
            .pipe(|_| Ok(()))
    }

    pub async fn get_clients_by_root<'a>(&'a self, root: &'a PathBuf) -> Vec<&'a NvimClient> {
        self.0
            .iter()
            .filter(|(_, client)| client.roots.contains(root))
            .map(|(_, client)| client)
            .collect()
    }

    pub async fn log_info(&self, root: &PathBuf, scope: &str, msg: &str) {
        for client in self.get_clients_by_root(&root).await {
            client.log_info(scope, msg).await.ok();
        }
    }

    pub async fn echo_msg(&self, root: &PathBuf, scope: &str, msg: &str) {
        let msg = format!("echo '{scope}: {msg}'");
        for client in self.get_clients_by_root(&root).await {
            client.echo_msg(&msg).await.ok();
        }
    }

    pub async fn echo_err(&self, root: &PathBuf, scope: &str, msg: &str) {
        let msg = format!("{scope}: {msg}");
        for client in self.get_clients_by_root(&root).await {
            client.echo_err(&msg).await.ok();
        }
    }

    pub async fn log_error(&self, root: &PathBuf, scope: &str, msg: &str) {
        for client in self.get_clients_by_root(&root).await {
            client.log_error(scope, msg).await.ok();
        }
    }

    pub async fn update_state(&self, update_state_script: &str) -> Result<()> {
        for (_, client) in self.iter() {
            client.sync_state(update_state_script).await?
        }
        Ok(())
    }
}
