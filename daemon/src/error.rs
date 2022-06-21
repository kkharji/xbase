use std::path::PathBuf;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("[Error] (Register): {0}")]
    Register(String),
    #[error("[Error] {0} is **not implemented**!")]
    NotImplemented(String),
    #[error("[Error] (Compile): {0}")]
    Compile(#[from] CompileError),
    #[error("{0}")]
    Tracing(#[from] log::TracingError),
    #[error("[Error] (Conversion) {0}")]
    Conversion(#[from] ConversionError),
    #[error("[Error] (Lookup) {0} with {0} doesn't exist")]
    NotFound(String, String),
    #[error("[Error] (Nvim) {0}")]
    Nvim(String),
    #[error("[Error] {0}")]
    Anyhow(#[from] anyhow::Error),
    #[error("[Error] (Loop) {0}")]
    Loop(#[from] LoopError),
    #[error("[Error] (IO) {0}")]
    IO(#[from] std::io::Error),
    #[error("[Error] (Conversion) convert string to value {0}")]
    Strum(#[from] strum::ParseError),
    #[error("[Error] (Conversion) convert using serde: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("[Error] (Conversion) convert using serde: {0}")]
    SerdeYaml(#[from] serde_yaml::Error),
    #[error("[Error] (Conversion) shell_words split: {0}")]
    Split(#[from] shell_words::ParseError),
    #[error("[Error] (Build) {0}")]
    Build(String),
    #[error("[Error] (Run) {0}")]
    Run(String),
    #[error("[Error] (Message) {0}")]
    Message(String),
    #[error("[Error] (Watcher) {0}")]
    NotifyWatch(#[from] notify::Error),
    #[error("[Error] (Which) {0}")]
    WhichError(#[from] which::Error),
    #[error("[Error]`{0}.Xcodeproj` Generate\n\n{1}")]
    XCodeProjectGenerate(String, String),
    #[error("[Error] {0}")]
    ProjectError(String),
}

#[derive(ThisError, Debug)]
pub enum CompileError {
    #[error("No compile commands generated for: {0:#?}")]
    Empty(PathBuf),
}

#[derive(ThisError, Debug)]
pub enum LoopError {
    #[error("No client found with {0} pid")]
    NoClient(i32),
    #[error("No project found with {0:?}")]
    NoProject(PathBuf),
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Self::Message(s)
    }
}

pub trait EnsureOptional<V> {
    fn to_result<T: std::fmt::Debug>(self, typ: &str, key: T) -> Result<V, Error>;
}

impl<V> EnsureOptional<V> for Option<V> {
    fn to_result<T: std::fmt::Debug>(self, typ: &str, key: T) -> Result<V, Error> {
        self.ok_or_else(|| Error::NotFound(typ.to_string(), format!("{key:?}")))
    }
}

#[derive(ThisError, Debug)]
pub enum ConversionError {
    #[error("Unable to convert path to string {0:?}")]
    PathToString(PathBuf),
    #[error("Unable to convert path to string {0:?}")]
    PathToFilename(PathBuf),
    #[error("Unable to convert value to string")]
    ToString,
}

impl From<simctl::Error> for Error {
    fn from(e: simctl::Error) -> Self {
        use tap::Pipe;
        match e {
            simctl::Error::Output { stderr, .. } => stderr
                .trim()
                .split(":")
                .skip(1)
                .collect::<String>()
                .replace("\n", " ")
                .trim()
                .pipe(|s| format!("{s}")),
            simctl::Error::Io(err) => err.to_string(),
            simctl::Error::Json(err) => err.to_string(),
            simctl::Error::Utf8(err) => err.to_string(),
        }
        .pipe(Self::Run)
    }
}

use nvim_rs::error::{CallError as NvimCallError, LoopError as NvimLoopError};

impl From<Box<NvimCallError>> for Error {
    fn from(e: Box<NvimCallError>) -> Self {
        use nvim_rs::error::*;
        use tap::Pipe;
        match &*e {
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
        }
        .pipe(Self::Nvim)
    }
}

impl From<NvimLoopError> for Error {
    fn from(e: NvimLoopError) -> Self {
        use nvim_rs::error::*;
        use tap::Pipe;
        match e {
            LoopError::MsgidNotFound(id) => format!("Message with id not found {id}"),
            LoopError::DecodeError(_, _) => format!("Unable to read message"),
            LoopError::InternalSendResponseError(_, res) => {
                format!("Unable to send message: {res:?}")
            }
        }
        .pipe(Self::Nvim)
    }
}
