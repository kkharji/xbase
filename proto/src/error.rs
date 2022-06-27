use serde::{Deserialize, Serialize};
use thiserror::Error as ThisError;

#[derive(Deserialize, Serialize)]
pub struct ErrorInner {
    kind: String,
    message: String,
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
    #[error("Failed to generate project definition: {0}")]
    Generate(String),
    #[error("Failed to parse project definition: {0}")]
    DefinitionParsing(String),
    #[error("No project definition found")]
    DefinitionLocating,
    #[error("Mutliple project found")]
    DefinitionMutliFound,
    #[error("{0}")]
    Unexpected(String),
    #[cfg(feature = "client")]
    #[error("{0}")]
    RPC(#[from] tarpc::client::RpcError),
    #[error("{0}")]
    JoinError(#[from] tokio::task::JoinError),
}

impl From<ErrorInner> for Error {
    fn from(v: ErrorInner) -> Self {
        match v.kind.as_str() {
            "Setup" => Self::Setup(v.message),
            "Build" => Self::Build(v.message),
            "Run" => Self::Run(v.message),
            "Generate" => Self::Generate(v.message),
            "DefinitionParsing" => Self::DefinitionParsing(v.message),
            "DefinitionLocating" => Self::DefinitionLocating,
            "DefinitionMutliFound" => Self::DefinitionMutliFound,
            _ => Self::Unexpected(v.message),
        }
    }
}

impl From<&Error> for ErrorInner {
    fn from(err: &Error) -> Self {
        let mut res = ErrorInner {
            kind: Default::default(),
            message: err.to_string(),
        };
        match err {
            Error::Setup(_) => res.kind = "Setup".into(),
            Error::Lookup(_, _) => res.kind = "Lookup".into(),
            Error::Build(_) => res.kind = "Build".into(),
            Error::Run(_) => res.kind = "Run".into(),
            Error::Generate(_) => res.kind = "Generate".into(),
            Error::DefinitionParsing(_) => res.kind = "DefinitionParsing".into(),
            Error::DefinitionLocating => res.kind = "DefinitionLocating".into(),
            Error::DefinitionMutliFound => res.kind = "DefinitionMutliFound".into(),
            Error::Unexpected(_) => res.kind = "General".into(),
            #[cfg(feature = "client")]
            Error::RPC(_) => res.kind = "RPC".into(),
            Error::JoinError(_) => res.kind = "JoinError".into(),
        };
        res
    }
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        ErrorInner::from(self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Error {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let error_inner = ErrorInner::deserialize(deserializer)?;
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

#[cfg(feature = "server")]
impl From<log::SetGlobalDefaultError> for Error {
    fn from(error: log::SetGlobalDefaultError) -> Self {
        Self::Unexpected(error.to_string())
    }
}

#[cfg(feature = "server")]
impl From<which::Error> for Error {
    fn from(error: which::Error) -> Self {
        Self::Unexpected(error.to_string())
    }
}

#[cfg(feature = "server")]
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

#[cfg(feature = "server")]
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

#[cfg(feature = "server")]
use nvim_rs::error::{CallError as NvimCallError, LoopError as NvimLoopError};

#[cfg(feature = "server")]
impl From<Box<NvimCallError>> for Error {
    fn from(e: Box<NvimCallError>) -> Self {
        use nvim_rs::error::*;
        Self::Unexpected(match &*e {
            NvimCallError::SendError(e, method) => {
                let err_msg = match e {
                    EncodeError::BufferError(e) => e.to_string(),
                    EncodeError::WriterError(e) => e.to_string(),
                };
                format!("{method}(...): {err_msg}")
            }
            NvimCallError::DecodeError(ref e, method) => {
                let err_msg = e.to_string();
                format!("{method}(...): {err_msg}")
            }
            NvimCallError::NeovimError(code, method) => {
                format!("{method}(...): error code {code:?}")
            }
            NvimCallError::WrongValueType(v) => format!("Wrong Value Type: {v:?}"),
            _ => e.to_string(),
        })
    }
}

#[cfg(feature = "server")]
impl From<NvimLoopError> for Error {
    fn from(e: NvimLoopError) -> Self {
        use nvim_rs::error::*;
        Self::Unexpected(match e {
            LoopError::MsgidNotFound(id) => format!("Message with id not found {id}"),
            LoopError::DecodeError(_, _) => format!("Unable to read message"),
            LoopError::InternalSendResponseError(_, res) => {
                format!("Unable to send message: {res:?}")
            }
        })
    }
}

#[cfg(feature = "neovim")]
impl From<Error> for mlua::Error {
    fn from(err: Error) -> Self {
        Self::external(err)
    }
}

#[cfg(feature = "neovim")]
impl From<mlua::Error> for Error {
    fn from(err: mlua::Error) -> Self {
        Self::Unexpected(err.to_string())
    }
}
