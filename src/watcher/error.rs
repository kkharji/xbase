use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum WatchError {
    Stop(String),
    Continue(String),
    FailToStart,
}

impl WatchError {
    pub fn stop(err: anyhow::Error) -> WatchError {
        Self::Stop(format!("WatchStop: {err}"))
    }

    pub fn r#continue(err: anyhow::Error) -> WatchError {
        Self::Continue(format!("WatchStop: {err}"))
    }
}
