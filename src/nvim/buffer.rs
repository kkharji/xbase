#[cfg(feature = "daemon")]
use super::NvimClient;
#[cfg(feature = "daemon")]
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, strum::EnumString, Serialize, Deserialize)]
#[strum(ascii_case_insensitive)]
pub enum BufferDirection {
    Float,
    Vertical,
    Horizontal,
}

#[cfg(feature = "daemon")]
impl BufferDirection {
    pub fn to_nvim_command(&self, bufnr: i64) -> String {
        match self {
            // TOOD: support build log float
            BufferDirection::Float => format!("sbuffer {bufnr}"),
            BufferDirection::Vertical => format!("vert sbuffer {bufnr}"),
            BufferDirection::Horizontal => format!("sbuffer {bufnr}"),
        }
    }

    pub async fn get_window_direction(
        nvim: &NvimClient,
        direction: Option<BufferDirection>,
        bufnr: i64,
    ) -> Result<String> {
        use std::str::FromStr;
        use tap::Pipe;

        if let Some(direction) = direction {
            return Ok(direction.to_nvim_command(bufnr));
        };

        "return require'xbase.config'.values.default_log_buffer_direction"
            .pipe(|str| nvim.exec_lua(str, vec![]))
            .await?
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Unable to covnert value to string"))?
            .pipe(BufferDirection::from_str)
            .map(|d| d.to_nvim_command(bufnr))
            .context("Convert to string to direction")
    }
}