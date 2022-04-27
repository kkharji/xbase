#![allow(dead_code)]
use std::{ops, path::Path};

use anyhow::Result;
use nvim_rs::{compat::tokio::Compat, create, error::LoopError, rpc::handler::Dummy, Neovim};
use parity_tokio_ipc::Connection;
use tokio::{io::WriteHalf, task::JoinHandle};

pub struct Nvim(
    Neovim<Compat<WriteHalf<Connection>>>,
    JoinHandle<Result<(), Box<LoopError>>>,
);

impl std::fmt::Debug for Nvim {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Nvim").finish()
    }
}

impl Nvim {
    pub async fn new<P: AsRef<Path> + Clone>(address: P) -> Result<Self> {
        let (neovim, handler) = create::tokio::new_path(address, Dummy::new()).await?;
        Ok(Self(neovim, handler))
    }

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

impl ops::Deref for Nvim {
    type Target = Neovim<Compat<WriteHalf<Connection>>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[tokio::test]
async fn test_current() {
    let path = "/var/folders/lm/jgnf6c7941qbrz4r6j5qscx00000gn/T/nvimXRp2Aj/0";
    let neovim = Nvim::new(path).await.unwrap();

    neovim
        .log_error("Build", "We Can't process your request")
        .await
        .unwrap();
}
