#[cfg(feature = "daemon")]
use {
    crate::{compile, nvim::NvimClient, state::State, util::fs::get_dirname_dir_root, Result},
    std::path::PathBuf,
    tokio::sync::MutexGuard,
};

use crate::types::Root;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct Client {
    pub pid: i32,
    pub root: Root,
    pub address: String,
}

#[cfg(feature = "daemon")]
impl Client {
    /// Get nvim client with state
    pub fn nvim<'a>(&self, state: &'a MutexGuard<'_, State>) -> Result<&'a NvimClient> {
        state.clients.get(&self.pid)
    }

    pub async fn echo_msg<'a>(&self, state: &'a MutexGuard<'_, State>, scope: &str, msg: &str) {
        state.clients.echo_msg(&self.root, scope, msg).await;
    }

    pub async fn echo_err<'a>(&self, state: &'a MutexGuard<'_, State>, scope: &str, msg: &str) {
        state.clients.echo_err(&self.root, scope, msg).await;
    }

    /// Check if client is registered in state
    pub fn is_registered<'a>(&self, state: &'a MutexGuard<'_, State>) -> bool {
        state.clients.contains_key(&self.pid)
    }

    /// Remove client from state
    pub fn remove_self<'a>(&self, state: &'a mut MutexGuard<'_, State>) {
        state.clients.remove(self);
    }

    /// Remove client from state
    pub async fn register_self<'a>(&self, state: &'a mut MutexGuard<'_, State>) -> Result<()> {
        state.clients.add(self).await
    }

    /// Register project if it's not already registered
    pub async fn register_project<'a>(&self, state: &'a mut MutexGuard<'_, State>) -> Result<()> {
        if let Ok(project) = state.projects.get_mut(&self.root) {
            project.clients.push(self.pid);
        } else {
            state.projects.add(self).await?;
            let ignore_pattern = state
                .projects
                .get(&self.root)
                .unwrap()
                .ignore_patterns
                .clone();

            state.watcher.add(self, ignore_pattern).await?;
        }

        Ok(())
    }

    /// Remove project root watcher
    pub fn remove_watcher<'a>(&self, state: &'a mut MutexGuard<'_, State>) {
        state.watcher.remove(self)
    }

    pub fn get_watcher_mut<'a>(
        &self,
        state: &'a mut MutexGuard<'_, State>,
    ) -> Result<&'a mut crate::watch::WatchService> {
        state.watcher.get_mut(&self.root)
    }

    pub fn get_watcher<'a>(
        &self,
        state: &'a mut MutexGuard<'_, State>,
    ) -> Result<&'a crate::watch::WatchService> {
        state.watcher.get(&self.root)
    }

    pub async fn ensure_server_support<'a>(
        &self,
        state: &'a mut MutexGuard<'_, State>,
        path: Option<&PathBuf>,
    ) -> Result<bool> {
        compile::ensure_server_support(state, self, path).await
    }
}

#[cfg(feature = "daemon")]
impl Client {
    pub fn abbrev_root(&self) -> String {
        get_dirname_dir_root(&self.root).unwrap_or_default()
    }
}

#[cfg(feature = "lua")]
use {crate::util::mlua::LuaExtension, mlua::prelude::*, tap::Pipe};

#[cfg(feature = "lua")]
impl Client {
    /// Derive client from lua value
    /// lua value can:
    /// - Client key with table value within it a key with "root"
    /// - Client key with string value representing "root"
    /// If value is none, then current working directory will be used
    /// lua value can either be a table with client key being a string
    pub fn derive(lua: &Lua, value: Option<LuaValue>) -> LuaResult<Self> {
        let root = match value {
            Some(LuaValue::Table(ref table)) => table.get("root")?,
            Some(LuaValue::String(ref root)) => root.to_string_lossy().to_string(),
            _ => lua.cwd()?,
        };
        Self {
            pid: std::process::id() as i32,
            address: lua.nvim_address()?,
            root: root.into(),
        }
        .pipe(Ok)
    }
}

#[cfg(feature = "lua")]
impl<'a> mlua::FromLua<'a> for Client {
    fn from_lua(value: mlua::Value<'a>, lua: &'a mlua::Lua) -> mlua::Result<Self> {
        Self::derive(
            lua,
            match value {
                LuaValue::Nil => None,
                _ => Some(value),
            },
        )
    }
}
