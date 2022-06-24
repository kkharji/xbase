use crate::nvim::NvimClient;
use crate::Result;
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use tap::Pipe;
use xbase_proto::{Client, IntoResult};

#[derive(Default, Debug, Serialize, derive_deref_rs::Deref)]
pub struct ClientStore(HashMap<i32, NvimClient>);

impl ClientStore {
    pub async fn add(&mut self, client: &Client) -> Result<()> {
        NvimClient::new(client)
            .await?
            .pipe(|client| self.insert(client.pid, client));
        log::info!("[{:?}] added", client.pid);
        Ok(())
    }

    pub fn remove(&mut self, client: &Client) {
        log::info!("[{:?}] removed", client.pid);
        self.0.remove(&client.pid);
    }

    pub fn get(&self, pid: &i32) -> Result<&NvimClient> {
        let client = self.0.get(&pid).into_result("Client", pid)?;
        log::trace!("[{:?}] accessed", pid);
        Ok(client)
    }

    pub fn get_mut(&mut self, pid: &i32) -> Result<&mut NvimClient> {
        let client = self.0.get_mut(&pid).into_result("Client", pid)?;
        log::trace!("[{:?}] accessed", pid);
        Ok(client)
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
