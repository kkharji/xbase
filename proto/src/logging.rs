use crate::LoggingTaskStatus;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Logging Task
#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct LoggingTask {
    pub path: PathBuf,
    pub status: LoggingTaskStatus,
    pub purpose: String,
}

#[cfg(feature = "neovim")]
impl mlua::prelude::LuaUserData for LoggingTask {}

#[cfg(feature = "neovim")]
use process_stream::*;

#[cfg(feature = "neovim")]
impl LoggingTask {
    async fn stream_logging_task(&self) -> crate::Result<ProcessStream> {
        let mut lines = linemux::MuxedLines::new()?;
        lines.add_file(&self.path).await?;
        Ok(stream! {
            while let Ok(Some(item)) = lines.next_line().await {
                let line = item.line();
                if let Ok(value) = serde_json::from_str::<ProcessItem>(&line){
                    if value == ProcessItem::Output("-LOGCLOSED-".into()) {
                        break;
                    }
                    yield value
                }
            }
        }
        .boxed())
    }

}
