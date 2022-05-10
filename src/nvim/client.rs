use super::*;

#[cfg(feature = "daemon")]
pub struct Nvim {
    pub nvim: NvimConnection,
    pub buffers: Buffers,
}

#[cfg(feature = "daemon")]
impl std::fmt::Debug for Nvim {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Nvim").finish()
    }
}

#[cfg(feature = "daemon")]
impl std::ops::Deref for Nvim {
    type Target = NvimConnection;
    fn deref(&self) -> &Self::Target {
        &self.nvim
    }
}

#[cfg(feature = "daemon")]
impl Nvim {
    pub async fn new<P>(address: P) -> Result<Self>
    where
        P: AsRef<Path> + Clone,
    {
        let (nvim, _) = connect(address, Dummy::new()).await?;
        let buffers = Buffers {
            log: NvimLogBuffer::new(&nvim).await?,
        };

        Ok(Self { nvim, buffers })
    }

    pub async fn run_lua(&self, script: &String) -> Result<()> {
        self.exec_lua(&script, vec![]).await.context("exec_lua")?;
        Ok(())
    }

    pub async fn sync_state_ws(ws: &Workspace) -> Result<()> {
        use crate::state::*;
        sync_project_state_ws(ws).await?;
        sync_is_watching_ws(ws).await?;
        Ok(())
    }
}

#[cfg(feature = "daemon")]
impl Nvim {
    async fn log(&self, level: &str, scope: &str, value: impl ToString) -> Result<()> {
        for line in value.to_string().split("\n") {
            let msg = format!(
                r#"require'xcodebase.log'.{level}("[{scope}]: {}")"#,
                line.escape_default()
            );
            self.exec_lua(&msg, Vec::default()).await?;
        }

        Ok(())
    }
    pub async fn log_info(&self, scope: &str, msg: impl ToString) -> Result<()> {
        self.log("info", scope, msg).await
    }
    pub async fn log_debug(&self, scope: &str, msg: impl ToString) -> Result<()> {
        self.log("debug", scope, msg).await
    }
    pub async fn log_error(&self, scope: &str, msg: impl ToString) -> Result<()> {
        self.log("error", scope, msg).await
    }
    pub async fn log_trace(&self, scope: &str, msg: impl ToString) -> Result<()> {
        self.log("trace", scope, msg).await
    }
    pub async fn log_warn(&self, scope: &str, msg: impl ToString) -> Result<()> {
        self.log("warn", scope, msg).await
    }
}

#[cfg(feature = "daemon")]
pub type ClientInner = std::collections::HashMap<i32, Nvim>;

#[cfg(feature = "daemon")]
#[derive(Debug, Default)]
pub struct NvimClients(String, ClientInner);

#[cfg(feature = "daemon")]
impl std::ops::Deref for NvimClients {
    type Target = ClientInner;
    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

#[cfg(feature = "daemon")]
impl std::ops::DerefMut for NvimClients {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.1
    }
}

#[cfg(feature = "daemon")]
impl NvimClients {
    pub fn new(name: String) -> Self {
        Self(name, Default::default())
    }

    pub async fn log_info(&self, msg: &str) {
        tracing::info!("{msg}");
        for (pid, nvim) in self.iter() {
            if let Err(e) = nvim.log_info(msg, false).await {
                tracing::error!("Fail to echo message to nvim clients ({pid}): {e}")
            }
        }
    }

    pub async fn log_error(&self, msg: &str) {
        tracing::error!("{msg}");
        for (pid, nvim) in self.iter() {
            if let Err(e) = nvim.log_error(msg, false).await {
                tracing::error!("Fail to echo message to nvim clients ({pid}): {e}")
            }
        }
    }

    /// Get nvim client
    pub fn get(&self, pid: &i32) -> Result<&Nvim> {
        match self.1.get(pid) {
            Some(o) => Ok(o),
            None => anyhow::bail!("No nvim instance for {pid}"),
        }
    }

    pub async fn insert<P>(&mut self, pid: i32, address: P) -> Result<()>
    where
        P: AsRef<Path> + Clone,
    {
        tracing::info!("[{}] New: {pid}", self.0);
        self.1.insert(pid, Nvim::new(address).await?);
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Client {
    pub pid: i32,
    pub root: String,
}

#[cfg(feature = "lua")]
impl<'a> mlua::FromLua<'a> for Client {
    fn from_lua(_lua_value: mlua::Value<'a>, lua: &'a mlua::Lua) -> mlua::Result<Self> {
        Ok(Self {
            pid: std::process::id() as i32,
            root: crate::util::mlua::LuaExtension::cwd(lua)?,
        })
    }
}
