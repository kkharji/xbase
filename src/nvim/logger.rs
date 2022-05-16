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
    current_line_count: Option<i64>,
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
            current_line_count: None,
        }
    }

    pub async fn log(&mut self, msg: String) -> Result<()> {
        let c = self.line_count().await?;

        self.buf.set_lines(c, c + 1, false, vec![msg]).await?;
        self.current_line_count = Some(c + 1);

        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        self.buf.set_lines(0, -1, false, vec![]).await?;
        Ok(())
    }

    pub async fn log_stream<S>(&mut self, mut stream: S, clear: bool, open: bool) -> Result<()>
    where
        S: Stream<Item = String> + Unpin,
    {
        self.set_running().await?;

        let title = format!(
            "[{}] ------------------------------------------------------",
            self.title
        );

        // TODO(nvim): close log buffer if it is open for new direction
        //
        // Currently the buffer direction will be ignored if the buffer is opened already

        if clear {
            self.clear().await?;
        }

        // TODO(nvim): build log correct height
        let win = if open {
            Some(self.open_win().await?)
        } else {
            None
        };

        self.log(title).await?;

        let mut success = false;
        while let Some(line) = stream.next().await {
            line.contains("Succeed").then(|| success = true);
            self.log(line).await?;
            if open {
                win.as_ref()
                    .unwrap()
                    .set_cursor((self.line_count().await?, 0))
                    .await?;
            }
        }

        self.set_status_end(success, open).await?;

        Ok(())
    }

    pub async fn set_status_end(&mut self, success: bool, open: bool) -> Result<()> {
        if success {
            self.nvim
                .exec("let g:xbase_watch_build_status='success'", false)
                .await?;
        } else {
            self.nvim
                .exec("let g:xbase_watch_build_status='failure'", false)
                .await?;
            if !open {
                self.nvim.exec(&self.open_cmd().await, false).await?;
                self.nvim
                    .get_current_win()
                    .await?
                    .set_cursor((self.line_count().await?, 0))
                    .await?;
                self.nvim.exec("call feedkeys('zt')", false).await?;
            }
        }
        Ok(())
    }

    pub async fn set_running(&mut self) -> Result<()> {
        self.nvim
            .exec("let g:xbase_watch_build_status='running'", false)
            .await?;
        Ok(())
    }

    pub async fn line_count(&'a self) -> Result<i64> {
        Ok(if let Some(count) = self.current_line_count {
            count
        } else {
            match self.buf.line_count().await? {
                1 => 0,
                count => count,
            }
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
