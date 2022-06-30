use process_stream::ProcessItem;
use serde::{Deserialize, Serialize};

/// Representation of Messages that clients needs to process
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Message {
    /// Notify use with a message
    Notify {
        msg: String,
        level: MessageLevel,
        pid: Option<i32>,
    },
    /// Log a message
    Log {
        msg: String,
        level: MessageLevel,
        pid: Option<i32>,
    },
    /// Execute an task
    Execute { task: Task, pid: Option<i32> },
}

impl Message {
    /// Whether the message should be skipped based on client pid.
    ///
    /// If pid is none then it always return false,
    /// if some and pid == other, then it will returns false,
    /// when the message pid != other then returns true
    pub fn should_skip(&self, other: u32) -> bool {
        match self {
            Message::Notify { pid, .. } => pid,
            Message::Log { pid, .. } => pid,
            Message::Execute { pid, .. } => pid,
        }
        .map(|pid| pid != other as i32)
        .unwrap_or_default()
    }
}

impl Message {
    pub fn notify_error<S: AsRef<str>>(value: S) -> Self {
        Self::Notify {
            msg: value.as_ref().to_string(),
            level: MessageLevel::Error,
            pid: None,
        }
    }

    pub fn notify_warn<S: AsRef<str>>(value: S) -> Self {
        Self::Notify {
            msg: value.as_ref().to_string(),
            level: MessageLevel::Warn,
            pid: None,
        }
    }

    pub fn notify_trace<S: AsRef<str>>(value: S) -> Self {
        Self::Notify {
            msg: value.as_ref().to_string(),
            level: MessageLevel::Trace,
            pid: None,
        }
    }

    pub fn notify_debug<S: AsRef<str>>(value: S) -> Self {
        Self::Notify {
            msg: value.as_ref().to_string(),
            level: MessageLevel::Debug,
            pid: None,
        }
    }

    pub fn log_error<S: AsRef<str>>(value: S) -> Self {
        Self::Log {
            msg: value.as_ref().to_string(),
            level: MessageLevel::Error,
            pid: None,
        }
    }

    pub fn log_info<S: AsRef<str>>(value: S) -> Self {
        Self::Log {
            msg: value.as_ref().to_string(),
            level: MessageLevel::Error,
            pid: None,
        }
    }

    pub fn log_warn<S: AsRef<str>>(value: S) -> Self {
        Self::Log {
            msg: value.as_ref().to_string(),
            level: MessageLevel::Warn,
            pid: None,
        }
    }

    pub fn log_trace<S: AsRef<str>>(value: S) -> Self {
        Self::Log {
            msg: value.as_ref().to_string(),
            level: MessageLevel::Trace,
            pid: None,
        }
    }

    pub fn log_debug<S: AsRef<str>>(value: S) -> Self {
        Self::Log {
            msg: value.as_ref().to_string(),
            level: MessageLevel::Debug,
            pid: None,
        }
    }
}

/// Statusline state
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum StatuslineState {
    /// Last task was successful
    Success,
    /// Last task failed
    Failure,
    /// A Request is being processed.
    Processing,
    /// Something is being watched.
    Watching,
    ///  that is currently running.
    Running,
}

/// Tasks that the clients should execute
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Task {
    UpdateStatusline(StatuslineState),
}

/// Message Kind
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum MessageLevel {
    /// Trace Message
    Trace = 0,
    /// Debug Message
    Debug = 1,
    /// Info Message
    Info = 2,
    /// Warn Message
    Warn = 3,
    /// Error Message
    Error = 4,
}

impl MessageLevel {
    /// Returns `true` if the message level is [`Trace`].
    ///
    /// [`Trace`]: MessageLevel::Trace
    #[must_use]
    pub fn is_trace(&self) -> bool {
        matches!(self, Self::Trace)
    }

    /// Returns `true` if the message level is [`Debug`].
    ///
    /// [`Debug`]: MessageLevel::Debug
    #[must_use]
    pub fn is_debug(&self) -> bool {
        matches!(self, Self::Debug)
    }

    /// Returns `true` if the message level is [`Info`].
    ///
    /// [`Info`]: MessageLevel::Info
    #[must_use]
    pub fn is_info(&self) -> bool {
        matches!(self, Self::Info)
    }

    /// Returns `true` if the message level is [`Warn`].
    ///
    /// [`Warn`]: MessageLevel::Warn
    #[must_use]
    pub fn is_warn(&self) -> bool {
        matches!(self, Self::Warn)
    }

    /// Returns `true` if the message level is [`Error`].
    ///
    /// [`Error`]: MessageLevel::Error
    #[must_use]
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error)
    }
}

impl From<ProcessItem> for Message {
    fn from(item: ProcessItem) -> Self {
        let is_success = item.is_success();
        match item {
            ProcessItem::Output(value) => {
                if value.to_lowercase().contains("error") {
                    Self::Log {
                        msg: value,
                        level: MessageLevel::Error,
                        pid: None,
                    }
                } else if value.to_lowercase().contains("warn") {
                    Self::Log {
                        msg: value,
                        level: MessageLevel::Warn,
                        pid: None,
                    }
                } else {
                    Self::Log {
                        msg: if value == "Resolving Packages" {
                            Default::default()
                        } else {
                            value
                        },
                        level: MessageLevel::Info,
                        pid: None,
                    }
                }
            }
            ProcessItem::Error(value) => Self::Log {
                msg: value,
                level: MessageLevel::Error,
                pid: None,
            },
            ProcessItem::Exit(code) => {
                if is_success.unwrap() {
                    Self::Log {
                        msg: Default::default(),
                        level: MessageLevel::Info,
                        pid: None,
                    }
                } else {
                    Self::Log {
                        msg: format!("Failed with {code} code"),
                        level: MessageLevel::Error,
                        pid: None,
                    }
                }
            }
        }
    }
}

impl From<String> for Message {
    fn from(value: String) -> Self {
        Self::Notify {
            msg: value,
            level: MessageLevel::Info,
            pid: None,
        }
    }
}

impl From<&str> for Message {
    fn from(value: &str) -> Self {
        Self::Notify {
            msg: value.to_string(),
            level: MessageLevel::Info,
            pid: None,
        }
    }
}
