mod buffer;
#[cfg(feature = "daemon")]
mod logger;

pub use buffer::BufferDirection;
use serde::{Deserialize, Serialize};

#[cfg(feature = "daemon")]
use {
    crate::{types::Client, util::fmt, Result},
    nvim_rs::{compat::tokio::Compat, create::tokio::new_path as connect, rpc::handler::Dummy},
};

#[cfg(feature = "daemon")]
pub use logger::*;

#[cfg(feature = "daemon")]
type NvimConnection = Compat<tokio::io::WriteHalf<parity_tokio_ipc::Connection>>;
#[cfg(feature = "daemon")]
pub type NvimWindow = nvim_rs::Window<NvimConnection>;

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
    pub async fn new(client: &Client) -> Result<Self> {
        let Client { root, pid, address } = client;
        let (nvim, _) = connect(address, Dummy::new()).await?;
        let buf = nvim.create_buf(false, true).await?;
        let log_bufnr = buf.get_number().await?;
        let script = format!("let g:xbase_log_bufnr={log_bufnr}");

        let (a, b, c) = tokio::join!(
            buf.set_name("[xbase Logs]"),
            buf.set_option("filetype", "xcodebuildlog".into()),
            nvim.exec(&script, false)
        );
        _ = (a?, b?, c?);

        Ok(NvimClient {
            pid: *pid,
            roots: vec![root.to_path_buf()],
            conn: nvim.into(),
            log_bufnr,
        })
    }

    pub async fn sync_state(&self, update_state_script: &str) -> Result<()> {
        self.exec_lua(update_state_script, vec![]).await?;
        Ok(())
    }

    pub fn new_logger<'a>(
        &'a self,
        ops: &str,
        target: &str,
        direction: &'a Option<BufferDirection>,
    ) -> Logger<'a> {
        Logger::new(
            self,
            fmt::as_section(format!("{ops}: {target}")),
            direction.clone(),
        )
    }

    pub fn new_unamed_logger<'a>(&'a self) -> Logger<'a> {
        Logger::new(self, "".into(), None)
    }
}

#[cfg(feature = "daemon")]
impl NvimClient {
    async fn log(&self, level: &str, scope: &str, value: impl ToString) -> Result<()> {
        for line in value.to_string().split("\n") {
            let msg = format!(
                r#"require'xbase.log'.{level}("[{scope}]: {}")"#,
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
    pub async fn echo_msg(&self, msg: &str) -> Result<()> {
        self.exec(msg, false).await?;
        Ok(())
    }
    pub async fn echo_err(&self, msg: &str) -> Result<()> {
        Ok(self.err_writeln(msg).await?)
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
