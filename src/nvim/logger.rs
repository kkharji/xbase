use super::{NvimClient, NvimConnection, NvimWindow};
use crate::nvim::BufferDirection;
use crate::Result;
use nvim_rs::{Buffer, Window};

pub struct Logger<'a> {
    pub nvim: &'a NvimClient,
    pub title: String,
    pub buf: Buffer<NvimConnection>,
    open_cmd: Option<String>,
    current_line_count: Option<i64>,
}

impl<'a> Logger<'a> {
    pub fn new(nvim: &'a NvimClient, title: String, direction: Option<BufferDirection>) -> Self {
        let buf = Buffer::new(nvim.log_bufnr.into(), nvim.inner().clone());
        let open_cmd = direction.map(|v| v.to_nvim_command(nvim.log_bufnr));

        Self {
            nvim,
            title,
            buf,
            open_cmd,
            current_line_count: None,
        }
    }

    pub async fn clear_content(&self) -> Result<()> {
        self.buf.set_lines(0, -1, false, vec![]).await?;
        Ok(())
    }

    async fn get_line_count(&'a self) -> Result<i64> {
        Ok(if let Some(count) = self.current_line_count {
            count
        } else {
            match self.buf.line_count().await? {
                1 => 0,
                count => count,
            }
        })
    }

    pub async fn log(&mut self, msg: String) -> Result<()> {
        tracing::debug!("{msg}");

        let mut c = self.get_line_count().await?;
        let lines = msg
            .split("\n")
            .map(ToString::to_string)
            .collect::<Vec<String>>();
        let lines_len = lines.len() as i64;

        self.buf
            .set_lines(c, c + lines_len as i64, false, lines)
            .await?;
        c += lines_len;

        self.current_line_count = Some(c);

        if let Some(win) = self.win().await {
            win.set_cursor((c, 0)).await?;
        }

        Ok(())
    }

    pub async fn log_title(&mut self) -> Result<()> {
        self.log(self.title.clone()).await?;
        Ok(())
    }

    /// Get window if it's available
    pub async fn win(&self) -> Option<NvimWindow> {
        let windows = self.nvim.list_wins().await.ok()?;
        for win in windows.into_iter() {
            let buf = win.get_buf().await.ok()?;
            if buf.get_number().await.ok()? == self.nvim.log_bufnr {
                return Some(win);
            }
        }
        None
    }

    /// Open Window
    pub async fn open_win(&self) -> Result<Window<NvimConnection>> {
        if let Some(win) = self.win().await {
            return Ok(win);
        }

        tracing::info!("Openning a new window");

        let open_cmd = match self.open_cmd.as_ref() {
            Some(s) => s.clone(),
            None => {
                BufferDirection::get_window_direction(self.nvim, None, self.nvim.log_bufnr).await?
            }
        };
        self.nvim.exec(&open_cmd, false).await?;
        let win = self.nvim.get_current_win().await?;
        // NOTE: This doesn't work
        win.set_option("number", false.into()).await?;
        win.set_option("relativenumber", false.into()).await?;
        // self.nvim.exec("setl nu nornu so=9", false).await?;
        self.nvim.exec("wincmd w", false).await?;

        Ok(win)
    }

    // TODO(logger): make running different for devices and app
    pub async fn set_running(&mut self) -> Result<()> {
        self.nvim
            .exec("let g:xbase_watch_build_status='running'", false)
            .await?;
        Ok(())
    }

    pub async fn set_status_end(&mut self, success: bool, open: bool) -> Result<()> {
        let win = self.win().await;
        if success {
            self.nvim
                .exec("let g:xbase_watch_build_status='success'", false)
                .await?;
        } else {
            self.nvim
                .exec("let g:xbase_watch_build_status='failure'", false)
                .await?;
        }

        if open && win.is_none() {
            self.open_win().await?;
            self.nvim.exec("call feedkeys('zt')", false).await?;
        }

        Ok(())
    }
}
