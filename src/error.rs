use std::path::PathBuf;
use thiserror::Error as ThisError;
use xcodebuild::parser::Step;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("[Error] (Compile): {0}")]
    Compile(#[from] CompileError),
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
    #[error("[Error] (XcodeGen) {0}")]
    XcodeGen(#[from] XcodeGenError),
    #[error("[Error] (Run) {0}")]
    Run(String),
    #[error("[Error] (WatchError) {0}")]
    Watch(#[from] WatchError),
}

#[derive(ThisError, Debug)]
pub enum CompileError {
    #[error("No compile commands generated\n{0:#?}")]
    NoCompileCommandsGenerated(Vec<Step>),
}

#[derive(ThisError, Debug)]
pub enum LoopError {
    #[error("No client found with {0} pid")]
    NoClient(i32),
    #[error("No project found with {0:?}")]
    NoProject(PathBuf),
}

#[derive(ThisError, Debug)]
pub enum XcodeGenError {
    #[error("project.yml is not found")]
    NoProjectConfig,
    #[error("Fail to `{0}.Xcodeproj`")]
    XcodeProjUpdate(String),
}

#[derive(ThisError, Debug)]
pub enum WatchError {
    #[error("(Stop) -> {0}")]
    Stop(String),
    #[error("(Continue) -> {0}")]
    Continue(String),
    #[error("(FailToStart)")]
    FailToStart,
}

#[cfg(feature = "daemon")]
impl WatchError {
    pub fn stop(err: Error) -> WatchError {
        Self::Stop(format!("WatchStop: {err}"))
    }

    pub fn continue_err(err: Error) -> WatchError {
        Self::Continue(format!("WatchContinue: {err}"))
    }
}

impl From<Error> for WatchError {
    fn from(e: Error) -> Self {
        match &e {
            Error::NotFound(_, _) => WatchError::Stop(e.to_string()),
            Error::Loop(_) => WatchError::Stop(e.to_string()),
            Error::IO(e) => WatchError::Stop(e.to_string()),
            _ => WatchError::Continue(e.to_string()),
        }
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

#[cfg(feature = "daemon")]
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

#[cfg(feature = "daemon")]
use nvim_rs::error::{CallError as NvimCallError, LoopError as NvimLoopError};

#[cfg(feature = "daemon")]
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

#[cfg(feature = "daemon")]
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
