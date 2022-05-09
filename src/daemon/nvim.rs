#![allow(dead_code)]
use std::{ops, path::Path, str::FromStr};
use tap::Pipe;
use tokio_stream::{Stream, StreamExt};

use anyhow::{Context, Result};
use nvim_rs::{
    compat::tokio::Compat, create, error::LoopError, rpc::handler::Dummy, Buffer, Neovim, Window,
};
use parity_tokio_ipc::Connection;
use tokio::{io::WriteHalf, task::JoinHandle};

pub struct Nvim {
    pub nvim: Neovim<Compat<WriteHalf<Connection>>>,
    handler: JoinHandle<Result<(), Box<LoopError>>>,
    pub log_bufnr: i64,
}

impl Nvim {
    pub async fn new<P: AsRef<Path> + Clone>(address: P) -> Result<Self> {
        let (neovim, handler) = create::tokio::new_path(address, Dummy::new()).await?;
        let buf = neovim.create_buf(false, true).await?;

        buf.set_name("[Xcodebase Logs]").await?;
        buf.set_option("filetype", "xcodebuildlog".into()).await?;

        Ok(Self {
            nvim: neovim,
            handler,
            log_bufnr: buf.get_number().await?,
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

    pub async fn log_to_buffer(
        &self,
        title: &str,
        direction: Option<WindowType>,
        mut stream: impl Stream<Item = String> + Unpin,
        clear: bool,
        open: bool,
    ) -> Result<()> {
        self.exec("let g:xcodebase_watch_build_status='running'", false)
            .await?;
        let title = format!("[{title}] ------------------------------------------------------");
        let buf = Buffer::new(self.log_bufnr.into(), self.nvim.clone());

        if clear {
            buf.set_lines(0, -1, false, vec![]).await?;
        }

        let mut c = match buf.line_count().await? {
            1 => 0,
            count => count,
        };

        // TODO(nvim): build log correct height
        // TODO(nvim): make auto clear configurable
        let command = match self.get_window_direction(direction).await {
            Ok(open_command) => open_command,
            Err(e) => {
                tracing::error!("Unable to convert value to string {e}");
                WindowType::Horizontal.to_nvim_command(self.log_bufnr)
            }
        };
        let mut win: Option<Window<Compat<WriteHalf<Connection>>>> = None;
        if open {
            self.exec(&command, false).await?;
            self.exec("setl nu nornu so=9", false).await?;
            win = Some(self.get_current_win().await?);
            self.exec("wincmd w", false).await?;
        }

        buf.set_lines(c, c + 1, false, vec![title]).await?;
        c += 1;
        let mut success = false;

        while let Some(line) = stream.next().await {
            if line.contains("Succeed") {
                success = true;
            }
            buf.set_lines(c, c + 1, false, vec![line]).await?;
            c += 1;
            if open {
                win.as_ref().unwrap().set_cursor((c, 0)).await?;
            }
        }

        if success {
            self.exec("let g:xcodebase_watch_build_status='success'", false)
                .await?;
        } else {
            self.exec("let g:xcodebase_watch_build_status='failure'", false)
                .await?;
            if !open {
                self.exec(&command, false).await?;
                self.get_current_win().await?.set_cursor((c, 0)).await?;
                self.exec("call feedkeys('zt')", false).await?;
            }
        }

        Ok(())
    }

    async fn get_window_direction(&self, direction: Option<WindowType>) -> Result<String> {
        if let Some(direction) = direction {
            return Ok(direction.to_nvim_command(self.log_bufnr));
        };

        "return require'xcodebase.config'.values.default_log_buffer_direction"
            .pipe(|str| self.exec_lua(str, vec![]))
            .await?
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Unable to covnert value to string"))?
            .pipe(WindowType::from_str)
            .map(|d| d.to_nvim_command(self.log_bufnr))
            .context("Convert to string to direction")
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

#[derive(strum::EnumString)]
#[strum(ascii_case_insensitive)]
pub enum WindowType {
    Float,
    Vertical,
    Horizontal,
}

impl WindowType {
    fn to_nvim_command(&self, bufnr: i64) -> String {
        match self {
            // TOOD: support build log float
            WindowType::Float => format!("sbuffer {bufnr}"),
            WindowType::Vertical => format!("vert sbuffer {bufnr}"),
            WindowType::Horizontal => format!("sbuffer {bufnr}"),
        }
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
