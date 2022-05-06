#![allow(dead_code)]
use std::{ops, path::Path};
use tokio_stream::StreamExt;

use anyhow::Result;
use nvim_rs::{
    compat::tokio::Compat, create, error::LoopError, rpc::handler::Dummy, Buffer, Neovim,
};
use parity_tokio_ipc::Connection;
use tokio::{io::WriteHalf, task::JoinHandle};

pub enum WindowType {
    Float,
    Vertical,
    Horizontal,
}

pub struct Nvim {
    pub nvim: Neovim<Compat<WriteHalf<Connection>>>,
    handler: JoinHandle<Result<(), Box<LoopError>>>,
    pub log_bufnr: i64,
}

impl Nvim {
    pub async fn new<P: AsRef<Path> + Clone>(address: P) -> Result<Self> {
        let (neovim, handler) = create::tokio::new_path(address, Dummy::new()).await?;
        let buffer = neovim.create_buf(false, true).await?;
        Ok(Self {
            nvim: neovim,
            handler,
            log_bufnr: buffer.get_number().await?,
        })
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

    pub async fn log_to_buffer(
        &self,
        title: &str,
        direction: WindowType,
        mut stream: impl tokio_stream::Stream<Item = String> + Unpin,
        clear: bool,
    ) -> Result<()> {
        let title = format!("[ {title} ]: ----> ");
        let buffer = Buffer::new(self.log_bufnr.into(), self.nvim.clone());

        if clear {
            buffer.set_lines(0, -1, false, vec![]).await?;
        }

        let mut c = match buffer.line_count().await? {
            1 => 0,
            count => count,
        };

        // TODO(nvim): build log control what direction to open buffer
        // TODO(nvim): build log correct width
        // TODO(nvim): build log auto scroll
        let command = match direction {
            // TOOD: build log float
            WindowType::Float => format!("sbuffer {}", self.log_bufnr),
            WindowType::Vertical => format!("vert sbuffer {}", self.log_bufnr),
            WindowType::Horizontal => format!("sbuffer {}", self.log_bufnr),
        };

        self.exec(&command, false).await?;

        buffer.set_lines(c, c + 1, false, vec![title]).await?;
        c += 1;

        while let Some(line) = stream.next().await {
            buffer.set_lines(c, c + 1, false, vec![line]).await?;
            c += 1
        }

        Ok(())
    }
}

impl std::fmt::Debug for Nvim {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Nvim").finish()
    }
}

impl ops::Deref for Nvim {
    type Target = Neovim<Compat<WriteHalf<Connection>>>;
    fn deref(&self) -> &Self::Target {
        &self.nvim
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
