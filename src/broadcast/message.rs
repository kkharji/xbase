use crate::{Broadcast, BuildSettings};
use serde::{Deserialize, Serialize};
use typescript_definitions::TypeScriptify;

/// Representation of Messages that clients needs to process
#[derive(Debug, Clone, Serialize, TypeScriptify)]
#[serde(tag = "type", content = "args")]
pub enum Message {
    /// Notify use with a message
    Notify {
        content: String,
        level: ContentLevel,
    },
    Log {
        content: String,
        level: ContentLevel,
    },
    /// Open Logger
    OpenLogger,
    /// Reload Language server
    ReloadLspServer,
    /// Set Current Task
    SetCurrentTask {
        kind: TaskKind,
        target: String,
        status: TaskStatus,
    },
    /// Update Current Task
    UpdateCurrentTask {
        content: String,
        level: ContentLevel,
    },
    FinishCurrentTask {
        status: TaskStatus,
    },
    /// Notify client that something is being watched
    SetWatching {
        watching: bool,
        settings: BuildSettings,
    },
}

/// What kind of task is currently under progress?
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TypeScriptify)]
pub enum TaskKind {
    /// Build Task
    Build,
    /// Run Task
    Run,
    /// Compile Project (maybe setup)
    Compile,
    /// Generate xcodeproj
    Generate,
}

/// What the status of task is currently under progress?
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TypeScriptify)]
pub enum TaskStatus {
    /// Task Failed,
    Failed,
    /// Task Succeeded,
    Succeeded,
    /// Processing Task,
    Processing,
}

/// What a given content level is? for whether to log/show it
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, TypeScriptify)]
pub enum ContentLevel {
    /// Trace Message
    Trace,
    /// Debug Message
    Debug,
    /// Info Message
    Info,
    /// Warn Message
    Warn,
    /// Error Message
    Error,
}

impl From<String> for Message {
    fn from(value: String) -> Self {
        Self::Notify {
            content: value.into(),
            level: ContentLevel::Info,
        }
    }
}

impl From<&str> for Message {
    fn from(value: &str) -> Self {
        Self::Notify {
            content: value.to_string().into(),
            level: ContentLevel::Info,
        }
    }
}

impl Broadcast {
    /// Tell connected clients to open logger
    pub fn open_logger(&self) {
        self.tx.send(Message::OpenLogger).ok();
    }

    /// Tell connected clients to reload language server
    pub fn reload_lsp_server(&self) {
        self.tx.send(Message::ReloadLspServer).ok();
    }

    /// Notify client with a message
    pub fn info<S: AsRef<str>>(&self, msg: S) {
        let msg = msg.as_ref();
        self.tx.send(msg.into()).ok();
    }

    /// Notify client with an error message
    pub fn error<S: AsRef<str>>(&self, msg: S) {
        let msg = msg.as_ref();
        tracing::error!("{msg}");
        self.tx
            .send(Message::Notify {
                content: msg.to_string(),
                level: ContentLevel::Error,
            })
            .ok();
    }

    /// Notify client with a warn message
    pub fn warn<S: AsRef<str>>(&self, msg: S) {
        let msg = msg.as_ref();
        tracing::warn!("{msg}");
        self.tx
            .send(Message::Notify {
                content: msg.to_string(),
                level: ContentLevel::Warn,
            })
            .ok();
    }

    /// Notify client with a trace message
    pub fn trace<S: AsRef<str>>(&self, msg: S) {
        let msg = msg.as_ref();
        tracing::trace!("{msg}");
        self.tx
            .send(Message::Notify {
                content: msg.to_string(),
                level: ContentLevel::Trace,
            })
            .ok();
    }

    /// Notify client with a debug message
    pub fn debug<S: AsRef<str>>(&self, msg: S) {
        let msg = msg.as_ref();
        tracing::debug!("{msg}");
        self.tx
            .send(Message::Notify {
                content: msg.to_string(),
                level: ContentLevel::Debug,
            })
            .ok();
    }

    /// Notify client with a message
    pub fn log_info<S: AsRef<str>>(&self, msg: S) {
        let msg = msg.as_ref();
        self.tx
            .send(Message::Log {
                content: msg.into(),
                level: ContentLevel::Info,
            })
            .ok();
    }

    /// Notify client with an error message
    pub fn log_error<S: AsRef<str>>(&self, msg: S) {
        let msg = msg.as_ref();
        tracing::error!("{msg}");
        self.tx
            .send(Message::Log {
                content: msg.to_string(),
                level: ContentLevel::Error,
            })
            .ok();
    }

    /// Notify client with a warn message
    pub fn log_warn<S: AsRef<str>>(&self, msg: S) {
        let msg = msg.as_ref();
        tracing::warn!("{msg}");
        self.tx
            .send(Message::Log {
                content: msg.to_string(),
                level: ContentLevel::Warn,
            })
            .ok();
    }

    /// Notify client with a trace message
    pub fn log_trace<S: AsRef<str>>(&self, msg: S) {
        let msg = msg.as_ref();
        tracing::trace!("{msg}");
        self.tx
            .send(Message::Log {
                content: msg.to_string(),
                level: ContentLevel::Trace,
            })
            .ok();
    }

    /// Notify client with a debug message
    pub fn log_debug<S: AsRef<str>>(&self, msg: S) {
        let msg = msg.as_ref();
        tracing::debug!("{msg}");
        self.tx
            .send(Message::Log {
                content: msg.to_string(),
                level: ContentLevel::Debug,
            })
            .ok();
    }

    pub fn update_current_task<S: AsRef<str>>(&self, content: S, level: ContentLevel) {
        self.tx
            .send(Message::UpdateCurrentTask {
                content: content.as_ref().into(),
                level,
            })
            .ok();
    }

    pub fn finish_current_task(&self, success: bool) {
        self.tx
            .send(Message::FinishCurrentTask {
                status: if success {
                    TaskStatus::Succeeded
                } else {
                    TaskStatus::Failed
                },
            })
            .ok();
    }
}
