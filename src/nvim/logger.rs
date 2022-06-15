use super::{NvimClient, NvimConnection, NvimWindow};
use crate::Result;
use crate::{nvim::BufferDirection, util::fmt};
use futures::StreamExt;
use nvim_rs::{Buffer, Window};
use xclog::XCLogger;

pub struct Logger<'a> {
    pub nvim: &'a NvimClient,
    title: String,
    buf: Buffer<NvimConnection>,
    open_cmd: Option<String>,
    current_line_count: Option<i64>,
}

impl<'a> Logger<'a> {
    /// Set logger title
    pub fn set_title(&mut self, title: String) -> &mut Self {
        self.title = title;
        self
    }

    /// Clear logger content
    pub async fn clear_content(&self) -> Result<()> {
        self.buf.set_lines(0, -1, false, vec![]).await?;
        Ok(())
    }

    /// Set open direction for looger
    pub fn set_direction(&mut self, direction: &BufferDirection) -> &mut Self {
        self.open_cmd = Some(direction.to_nvim_command(self.nvim.log_bufnr));
        self
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

    /// Consume bulid logs via logging them to client
    pub async fn consume_build_logs(
        &mut self,
        mut xclogger: XCLogger,
        clear: bool,
        open: bool,
    ) -> Result<bool> {
        let mut success = true;
        // TODO(nvim): close log buffer if it is open for new direction
        // Currently the buffer direction will be ignored if the buffer is opened already
        if clear {
            self.clear_content().await?;
        }

        // TODO(nvim): build log correct height
        if open {
            self.open_win().await?;
        }

        self.set_running(false).await?;

        while let Some(line) = xclogger.next().await {
            line.contains("FAILED").then(|| success = false);

            self.append(line.to_string()).await?;
        }

        self.set_status_end(success, open).await?;

        Ok(success)
    }

    // TODO(logger): always show current new logs in middle of the window
    pub async fn append(&mut self, msg: String) -> Result<()> {
        tracing::trace!("{msg}");
        let win_info = self.win().await;
        let mut c = self.get_line_count().await?;

        let lines = msg
            .split("\n")
            .map(|s| format!("[{}] {}", self.title, s))
            .collect::<Vec<String>>();

        self.set_lines(&mut c, lines).await?;

        if let Some((focused, win)) = win_info {
            // self.nvim.exec("call feedkeys('zt')", false).await?;
            if !focused {
                win.set_cursor((c, 0)).await?;
            } else {
                let (current, _) = win.get_cursor().await?;
                let diff = c - current;
                if diff == 1 || diff == 2 {
                    // self.nvim.exec("call feedkeys('zt')", false).await?;
                    win.set_cursor((c, 0)).await?;
                }
            }
        }

        self.current_line_count = Some(c);

        Ok(())
    }

    async fn set_lines(&mut self, c: &mut i64, lines: Vec<String>) -> Result<()> {
        *c += lines.len() as i64;
        self.buf.set_lines(*c - 1, *c, false, lines).await?;
        Ok(())
    }

    /// Get logger window if it's available and whether is currently focused.
    pub async fn win(&self) -> Option<(bool, NvimWindow)> {
        let windows = self.nvim.list_wins().await.ok()?;
        for win in windows.into_iter() {
            let buf = win.get_buf().await.ok()?;

            if buf.get_number().await.ok()? == self.nvim.log_bufnr {
                let curr = self.nvim.get_current_win().await.ok()?;
                let is_focused = curr.get_number().await.ok()? == win.get_number().await.ok()?;
                return Some((is_focused, win));
            }
        }
        None
    }

    /// Open Window
    pub async fn open_win(&mut self) -> Result<Window<NvimConnection>> {
        if let Some((_, win)) = self.win().await {
            return Ok(win);
        }

        tracing::info!("Openning a new window");

        if self.open_cmd.is_none() {
            let v = self.nvim.get_window_direction(None).await?;
            self.open_cmd = Some(v);
        };

        // TODO(nvim): setup autocmd for buffer type
        let setup_script = format!(
            r#"
            {}
            setlocal nonumber norelativenumber
            setlocal scrolloff=3
            "#,
            self.open_cmd.as_ref().unwrap()
        );

        self.nvim.exec(&setup_script, false).await?;
        let win = self.nvim.get_current_win().await?;
        self.nvim.exec("wincmd w", false).await?;

        Ok(win)
    }

    pub async fn set_running(&mut self, is_device: bool) -> Result<()> {
        let var = if is_device {
            "let g:xbase_watch_build_status='device_running'"
        } else {
            "let g:xbase_watch_build_status='running'"
        };

        self.nvim.exec(var, false).await?;
        Ok(())
    }

    pub async fn set_status_end(&mut self, success: bool, open: bool) -> Result<()> {
        let win = self.win().await;
        if success {
            self.nvim
                .exec("let g:xbase_watch_build_status='success'", false)
                .await?;
            self.append(fmt::separator()).await?;
        } else {
            self.nvim
                .exec("let g:xbase_watch_build_status='failure'", false)
                .await?;
        }

        if open && win.is_none() || !success {
            self.open_win().await?;
            self.nvim.exec("call feedkeys('zt')", false).await?;
        }

        Ok(())
    }
}

impl NvimClient {
    pub fn logger<'a>(&'a self) -> Logger<'a> {
        Logger {
            nvim: self,
            title: Default::default(),
            buf: Buffer::new(self.log_bufnr.into(), self.inner().clone()),
            open_cmd: None,
            current_line_count: None,
        }
    }

    async fn get_window_direction(&self, direction: Option<BufferDirection>) -> Result<String> {
        use std::str::FromStr;
        use tap::Pipe;
        let ref bufnr = self.log_bufnr;

        if let Some(direction) = direction {
            return Ok(direction.to_nvim_command(*bufnr));
        };

        match "return require'xbase.config'.values.default_log_buffer_direction"
            .pipe(|str| self.exec_lua(str, vec![]))
            .await?
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Unable to covnert value to string"))?
            .pipe(BufferDirection::from_str)
            .map(|d| d.to_nvim_command(*bufnr))
        {
            Ok(open_command) => open_command,
            Err(e) => {
                tracing::error!("Unable to convert value to string {e}");
                BufferDirection::Horizontal.to_nvim_command(*bufnr)
            }
        }
        .pipe(Ok)
    }
}
