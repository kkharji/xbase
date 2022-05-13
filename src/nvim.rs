mod buffer;
#[cfg(feature = "daemon")]
mod watchlogger;

pub use buffer::BufferDirection;

#[cfg(feature = "daemon")]
use nvim_rs::compat::tokio::Compat;

#[cfg(feature = "daemon")]
pub use watchlogger::*;

use serde::{Deserialize, Serialize};

#[cfg(feature = "daemon")]
use anyhow::Result;

#[cfg(feature = "daemon")]
type NvimConnection = Compat<tokio::io::WriteHalf<parity_tokio_ipc::Connection>>;

#[derive(Deserialize, Serialize)]
pub struct NvimClient {
    pub pid: i32,
    pub roots: Vec<crate::types::Root>,
    #[cfg(feature = "daemon")]
    #[serde(skip)]
    pub conn: Option<nvim_rs::Neovim<NvimConnection>>,
    pub log_bufnr: i64,
}

#[cfg(feature = "daemon")]
impl NvimClient {
    pub async fn new(req: &crate::daemon::Register) -> Result<Self> {
        use nvim_rs::create::tokio::new_path as connect;
        use nvim_rs::rpc::handler::Dummy;

        let crate::daemon::Register { address, client } = req;
        let crate::types::Client { root, pid } = client;

        let (nvim, _) = connect(address, Dummy::new()).await?;
        let buf = nvim.create_buf(false, true).await?;
        let log_bufnr = buf.get_number().await?;

        buf.set_name("[Xcodebase Logs]").await?;
        buf.set_option("filetype", "xcodebuildlog".into()).await?;

        // NOTE: store log bufnr somewhere in vim state
        nvim.exec(&format!("let g:xcodebase_log_bufnr={log_bufnr}"), false)
            .await?;

        Ok(NvimClient {
            pid: *pid,
            roots: vec![root.to_path_buf()],
            conn: Some(nvim),
            log_bufnr,
        })
    }

    pub async fn sync_state(&self, update_state_script: &str) -> Result<()> {
        self.exec_lua(update_state_script, vec![]).await?;
        Ok(())
    }
}

#[cfg(feature = "daemon")]
impl NvimClient {
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

    fn inner(&self) -> &nvim_rs::Neovim<NvimConnection> {
        self.conn.as_ref().unwrap()
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
impl std::fmt::Debug for NvimClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NvimClient")
            .field("pid", &self.pid)
            .field("roots", &self.roots)
            .finish()
    }
}

#[cfg(feature = "daemon")]
impl std::ops::Deref for NvimClient {
    type Target = nvim_rs::Neovim<NvimConnection>;
    fn deref(&self) -> &Self::Target {
        &self.conn.as_ref().unwrap()
    }
}

#[cfg(feature = "daemon")]
impl std::ops::DerefMut for NvimClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.conn.as_mut().unwrap()
    }
}
