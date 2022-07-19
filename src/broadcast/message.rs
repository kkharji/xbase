use crate::{BuildSettings, ProjectInfo, Runners};
use serde::{Deserialize, Serialize};
use typescript_type_def::TypeDef;

/// State usesd to set client state
#[derive(Debug, Serialize, TypeDef)]
#[serde(tag = "key", content = "value")]
#[serde(rename_all = "camelCase")]
pub enum State {
    Runners(Runners),
    ProjectInfo(ProjectInfo),
}

/// Representation of Messages that clients needs to process
#[derive(Debug, Serialize, TypeDef)]
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
    /// Notification to client to update a state with the given value
    SetState(State),
    /// Internal!
    #[serde(skip)]
    Disconnect,
}

/// What kind of task is currently under progress?
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TypeDef)]
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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TypeDef)]
pub enum TaskStatus {
    /// Task Failed,
    Failed,
    /// Task Succeeded,
    Succeeded,
    /// Processing Task,
    Processing,
}

/// What a given content level is? for whether to log/show it
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, TypeDef)]
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
