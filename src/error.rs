use serde::{Deserialize, Serialize};
use thiserror::Error as ThisError;
use typescript_type_def::TypeDef;

/// Server Error due to failure while processing a `Request
#[derive(Debug, Deserialize, Serialize, TypeDef)]
pub struct ServerError {
    kind: String,
    msg: String,
}

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Failed to setup project: {0}")]
    Setup(String),
    #[error("No `{1}` found with `{0}`")]
    /// Key, Type
    Lookup(String, String),
    #[error("Failed to build target/scheme: {0}")]
    Build(String),
    #[error("Failed to run target/scheme: {0}")]
    Run(String),
    #[error("Failed to generate project definition")]
    Generate,
    #[error("Failed to generate compile commands")]
    Compile,
    #[error("Failed to parse project definition: {0}")]
    DefinitionParsing(String),
    #[error("No project definition found")]
    DefinitionLocating,
    #[error("Mutliple project found")]
    DefinitionMutliFound,
    #[error("{0}")]
    Unexpected(String),
    #[error("{0}")]
    JoinError(String),
    #[error("{0}")]
    SendError(String),
    #[error("Failed to parse broadcast message: {0}")]
    MessageParse(String),
}

impl From<ServerError> for Error {
    fn from(v: ServerError) -> Self {
        match v.kind.as_str() {
            "Setup" => Self::Setup(v.msg),
            "Build" => Self::Build(v.msg),
            "Run" => Self::Run(v.msg),
            "Generate" => Self::Generate,
            "DefinitionParsing" => Self::DefinitionParsing(v.msg),
            "DefinitionLocating" => Self::DefinitionLocating,
            "DefinitionMutliFound" => Self::DefinitionMutliFound,
            "SendError" => Self::SendError(v.msg),
            "MessageParse" => Self::MessageParse(v.msg),
            _ => Self::Unexpected(v.msg),
        }
    }
}

impl From<&Error> for ServerError {
    fn from(err: &Error) -> Self {
        let mut res = ServerError {
            kind: Default::default(),
            msg: err.to_string(),
        };
        match err {
            Error::Setup(_) => res.kind = "Setup".into(),
            Error::Lookup(_, _) => res.kind = "Lookup".into(),
            Error::Build(_) => res.kind = "Build".into(),
            Error::Run(_) => res.kind = "Run".into(),
            Error::Generate => res.kind = "Generate".into(),
            Error::DefinitionParsing(_) => res.kind = "DefinitionParsing".into(),
            Error::DefinitionLocating => res.kind = "DefinitionLocating".into(),
            Error::DefinitionMutliFound => res.kind = "DefinitionMutliFound".into(),
            Error::Unexpected(_) => res.kind = "General".into(),
            Error::JoinError(_) => res.kind = "JoinError".into(),
            Error::SendError(_) => res.kind = "SendError".into(),
            Error::MessageParse(_) => res.kind = "MessageParse".into(),
            Error::Compile => res.kind = "Compile".into(),
        };
        res
    }
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        ServerError::from(self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Error {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let error_inner = ServerError::deserialize(deserializer)?;
        Ok(error_inner.into())
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self::Unexpected(error.to_string())
    }
}

impl From<anyhow::Error> for Error {
    fn from(error: anyhow::Error) -> Self {
        Self::Unexpected(error.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Unexpected(error.to_string())
    }
}

impl From<strum::ParseError> for Error {
    fn from(error: strum::ParseError) -> Self {
        Self::Unexpected(error.to_string())
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Self::Unexpected(s)
    }
}

// impl From<tracing::> for Error {
//     fn from(error: tracing::SetGlobalDefaultError) -> Self {
//         Self::Unexpected(error.to_string())
//     }
// }

impl From<which::Error> for Error {
    fn from(error: which::Error) -> Self {
        Self::Unexpected(error.to_string())
    }
}

impl From<notify::Error> for Error {
    fn from(error: notify::Error) -> Self {
        Self::Unexpected(format!("Watcher: {error}"))
    }
}

/// Convert option into result
pub trait IntoResult<V> {
    fn into_result<T: std::fmt::Debug>(self, typ: &str, key: T) -> Result<V, Error>;
}

impl<V> IntoResult<V> for Option<V> {
    fn into_result<T: std::fmt::Debug>(self, typ: &str, key: T) -> Result<V, Error> {
        self.ok_or_else(|| Error::Lookup(typ.to_string(), format!("{key:?}")))
    }
}

impl From<simctl::Error> for Error {
    fn from(e: simctl::Error) -> Self {
        Self::Run(match e {
            simctl::Error::Output { stderr, .. } => stderr
                .trim()
                .split(":")
                .skip(1)
                .collect::<String>()
                .replace("\n", " ")
                .trim()
                .into(),
            simctl::Error::Io(err) => err.to_string(),
            simctl::Error::Json(err) => err.to_string(),
            simctl::Error::Utf8(err) => err.to_string(),
        })
    }
}

impl<T: std::fmt::Debug> From<tokio::sync::mpsc::error::SendError<T>> for Error {
    fn from(v: tokio::sync::mpsc::error::SendError<T>) -> Self {
        Self::SendError(format!("Channel closed, unable to send `{:?}`", v))
    }
}
