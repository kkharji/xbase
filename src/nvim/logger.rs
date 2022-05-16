use super::{NvimClient, NvimConnection};
use crate::nvim::BufferDirection;
use crate::types::BuildConfiguration;
use anyhow::Result;
use nvim_rs::{Buffer, Window};
use tokio_stream::{Stream, StreamExt};

pub struct Logger<'a> {
    pub nvim: &'a NvimClient,
    pub title: &'a str,
    pub request: &'a BuildConfiguration,
    pub buf: Buffer<NvimConnection>,
    open_cmd: Option<String>,
}

impl<'a> Logger<'a> {
    pub fn new(
        nvim: &'a NvimClient,
        title: &'a str,
        request: &'a BuildConfiguration,
        direction: Option<BufferDirection>,
    ) -> Self {
        let buf = Buffer::new(nvim.log_bufnr.into(), nvim.inner().clone());
        let open_cmd = direction.map(|v| v.to_nvim_command(nvim.log_bufnr));

        Self {
            nvim,
            title,
            request,
            buf,
            open_cmd,
        }
    }

    pub async fn log(&self, msg: String) -> Result<()> {
        let Self { buf, .. } = self;
        let c = self.line_count().await?;

        buf.set_lines(c, c + 1, false, vec![msg]).await?;

        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        self.buf.set_lines(0, -1, false, vec![]).await?;
        Ok(())
    }

    pub async fn log_stream<S>(&self, mut stream: S, clear: bool, open: bool) -> Result<()>
    where
        S: Stream<Item = String> + Unpin,
    {
        let Self {
            nvim,
            title,
            request,
            buf,
            ..
        } = self;

        let BuildConfiguration { .. } = request;

        nvim.exec("let g:xbase_watch_build_status='running'", false)
            .await?;

        let title = format!(
            "[{}] ------------------------------------------------------",
            title
        );

        // TODO(nvim): close log buffer if it is open for new direction
        //
        // Currently the buffer direction will be ignored if the buffer is opened already

        if clear {
            self.clear().await?;
        }

        let mut c = self.line_count().await?;

        // TODO(nvim): build log correct height
        let win = if open {
            Some(self.open_win().await?)
        } else {
            None
        };

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
            nvim.exec("let g:xbase_watch_build_status='success'", false)
                .await?;
        } else {
            nvim.exec("let g:xbase_watch_build_status='failure'", false)
                .await?;
            if !open {
                nvim.exec(&self.open_cmd().await, false).await?;
                nvim.get_current_win().await?.set_cursor((c, 0)).await?;
                nvim.exec("call feedkeys('zt')", false).await?;
            }
        }

        Ok(())
    }

    async fn line_count(&self) -> Result<i64> {
        Ok(match self.buf.line_count().await? {
            1 => 0,
            count => count,
        })
    }

    async fn get_open_cmd(&self, direction: Option<BufferDirection>) -> String {
        let direction =
            BufferDirection::get_window_direction(self.nvim, direction, self.nvim.log_bufnr);

        match direction.await {
            Ok(open_command) => open_command,
            Err(e) => {
                tracing::error!("Unable to convert value to string {e}");
                BufferDirection::Horizontal.to_nvim_command(self.nvim.log_bufnr)
            }
        }
    }

    pub async fn open_win(&self) -> Result<Window<NvimConnection>> {
        let open_cmd = self.open_cmd().await;

        self.nvim.exec(&open_cmd, false).await?;
        let win = self.nvim.get_current_win().await?;
        self.nvim.exec("setl nu nornu so=9", false).await?;
        self.nvim.exec("wincmd w", false).await?;

        Ok(win)
    }

    async fn open_cmd(&self) -> String {
        let open_cmd = match self.open_cmd.as_ref() {
            Some(s) => s.clone(),
            None => self.get_open_cmd(None).await,
        };
        open_cmd
    }
}
