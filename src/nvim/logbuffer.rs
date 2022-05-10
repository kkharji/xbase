use super::*;

pub struct NvimLogBuffer {
    pub bufnr: i64,
}

pub struct BulkLogRequest<'a, S: Stream<Item = String> + Unpin> {
    pub nvim: &'a Nvim,
    pub title: &'a str,
    pub direction: Option<BufferDirection>,
    pub stream: S,
    pub clear: bool,
    pub open: bool,
}

impl NvimLogBuffer {
    pub async fn new(nvim: &NvimConnection) -> Result<Self> {
        let buf = nvim.create_buf(false, true).await?;

        buf.set_name("[Xcodebase Logs]").await?;
        buf.set_option("filetype", "xcodebuildlog".into()).await?;
        // TODO: store log bufnr somewhere in vim state

        let bufnr = buf.get_number().await?;

        Self { bufnr }.pipe(Ok)
    }

    pub async fn bulk_append<'a, S: Stream<Item = String> + Unpin>(
        &self,
        mut req: BulkLogRequest<'a, S>,
    ) -> Result<()> {
        req.nvim
            .exec("let g:xcodebase_watch_build_status='running'", false)
            .await?;
        let title = format!(
            "[{}] ------------------------------------------------------",
            req.title
        );
        let buf = Buffer::new(self.bufnr.into(), req.nvim.nvim.clone());

        if req.clear {
            buf.set_lines(0, -1, false, vec![]).await?;
        }

        let mut c = match buf.line_count().await? {
            1 => 0,
            count => count,
        };

        // TODO(nvim): build log correct height
        // TODO(nvim): make auto clear configurable
        let command = match BufferDirection::get_window_direction(
            req.nvim,
            req.direction,
            self.bufnr,
        )
        .await
        {
            Ok(open_command) => open_command,
            Err(e) => {
                tracing::error!("Unable to convert value to string {e}");
                BufferDirection::Horizontal.to_nvim_command(self.bufnr)
            }
        };
        let mut win: Option<Window<Connection>> = None;
        if req.open {
            req.nvim.exec(&command, false).await?;
            req.nvim.exec("setl nu nornu so=9", false).await?;
            win = Some(req.nvim.get_current_win().await?);
            req.nvim.exec("wincmd w", false).await?;
        }

        buf.set_lines(c, c + 1, false, vec![title]).await?;
        c += 1;
        let mut success = false;

        while let Some(line) = req.stream.next().await {
            if line.contains("Succeed") {
                success = true;
            }
            buf.set_lines(c, c + 1, false, vec![line]).await?;
            c += 1;
            if req.open {
                win.as_ref().unwrap().set_cursor((c, 0)).await?;
            }
        }

        if success {
            req.nvim
                .exec("let g:xcodebase_watch_build_status='success'", false)
                .await?;
        } else {
            req.nvim
                .exec("let g:xcodebase_watch_build_status='failure'", false)
                .await?;
            if !req.open {
                req.nvim.exec(&command, false).await?;
                req.nvim.get_current_win().await?.set_cursor((c, 0)).await?;
                req.nvim.exec("call feedkeys('zt')", false).await?;
            }
        }

        Ok(())
    }
}
