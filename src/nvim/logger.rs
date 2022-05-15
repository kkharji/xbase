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
}

impl<'a> Logger<'a> {
    pub fn new(nvim: &'a NvimClient, title: &'a str, request: &'a BuildConfiguration) -> Self {
        Self {
            nvim,
            title,
            request,
        }
    }

    pub async fn log_stream<S>(
        &self,
        mut stream: S,
        direction: Option<BufferDirection>,
        clear: bool,
        open: bool,
    ) -> Result<()>
    where
        S: Stream<Item = String> + Unpin,
    {
        let Self {
            nvim,
            title,
            request,
        } = self;

        let BuildConfiguration { .. } = request;

        nvim.exec("let g:xbase_watch_build_status='running'", false)
            .await?;

        let title = format!(
            "[{}] ------------------------------------------------------",
            title
        );

        let buf = Buffer::new(nvim.log_bufnr.into(), nvim.inner().clone());
        // TODO(nvim): close log buffer if it is open for new direction
        //
        // Currently the buffer direction will be ignored if the buffer is opened already

        if clear {
            buf.set_lines(0, -1, false, vec![]).await?;
        }

        let mut c = match buf.line_count().await? {
            1 => 0,
            count => count,
        };

        // TODO(nvim): build log correct height
        let command =
            match BufferDirection::get_window_direction(nvim, direction, nvim.log_bufnr).await {
                Ok(open_command) => open_command,
                Err(e) => {
                    tracing::error!("Unable to convert value to string {e}");
                    BufferDirection::Horizontal.to_nvim_command(nvim.log_bufnr)
                }
            };

        let mut win: Option<Window<NvimConnection>> = None;

        if open {
            nvim.exec(&command, false).await?;
            nvim.exec("setl nu nornu so=9", false).await?;
            win = Some(nvim.get_current_win().await?);
            nvim.exec("wincmd w", false).await?;
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
            nvim.exec("let g:xbase_watch_build_status='success'", false)
                .await?;
        } else {
            nvim.exec("let g:xbase_watch_build_status='failure'", false)
                .await?;
            if !open {
                nvim.exec(&command, false).await?;
                nvim.get_current_win().await?.set_cursor((c, 0)).await?;
                nvim.exec("call feedkeys('zt')", false).await?;
            }
        }

        Ok(())
    }
}
