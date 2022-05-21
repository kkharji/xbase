//! Helper type for processing process output and exit status in non-blocking way
// use futures::io::Chain;
use futures::stream::{once, Stream, StreamExt};
use std::{fmt::Display, io};
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, BufReader},
    process::Child,
};

/// Create new process from [`Child`]
/// Child must set stdout and stderr or it might panic.
pub fn stream(mut child: Child) -> io::Result<impl Stream<Item = Output> + Send> {
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let status = tokio::spawn(async move { child.wait().await });

    fn to_output(v: io::Result<String>, is_stdout: bool) -> Output {
        match v {
            Ok(line) if is_stdout => Output::Out(line),
            Ok(line) => Output::Err(line),
            Err(e) => Output::Err(e.to_string()),
        }
    }

    use tokio_stream::wrappers::LinesStream;
    fn to_stream<R: AsyncRead>(out: R) -> LinesStream<BufReader<R>> {
        LinesStream::new(BufReader::new(out).lines())
    }

    let stdout_stream = to_stream(stdout).map(|v| to_output(v, true));
    let stderr_stream = to_stream(stderr).map(|v| to_output(v, false));
    let exit_stream = once(async move {
        match status.await {
            Err(err) => Output::Err(err.to_string()),
            Ok(Ok(status)) => Output::Exit(Ok(status.code())),
            Ok(Err(err)) => Output::Exit(Err(err)),
        }
    });
    let stream = tokio_stream::StreamExt::merge(stdout_stream, stderr_stream)
        .chain(exit_stream)
        .boxed();

    Ok(stream)
}

/// Output produced by [`process`]
#[derive(Debug)]
pub enum Output {
    /// Source stdout
    Out(String),
    /// Source stderr or internal io::Error
    Err(String),
    /// Exit status
    Exit(Result<Option<i32>, io::Error>),
}

impl Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Output::Out(msg) => msg.fmt(f),
            Output::Err(msg) => write!(f, "[Error] {msg}"),
            Output::Exit(Ok(Some(code))) => code.fmt(f),
            _ => Ok(()),
        }
    }
}
