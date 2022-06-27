use crate::Result;
use process_stream::*;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{
    fs::OpenOptions,
    io::AsyncWriteExt,
    sync::{mpsc::*, Notify},
    task::JoinHandle,
};
use xbase_proto::LoggingTask;

pub struct Logger {
    purpose: String,
    file_path: PathBuf,
    tx: UnboundedSender<ProcessItem>,
    abort: Arc<Notify>,
    handle: JoinHandle<()>,
}

impl Logger {
    pub async fn new<P: AsRef<Path>, S: AsRef<str>>(
        new_log_file_path: P,
        purpose: S,
    ) -> Result<Self> {
        let (tx, mut rx) = unbounded_channel();
        let abort = Arc::new(tokio::sync::Notify::new());
        let abort1 = abort.clone();
        let log_file_path = new_log_file_path.as_ref().to_path_buf();
        let log_file = tokio::fs::File::create(&log_file_path).await.unwrap();
        drop(log_file);

        let handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = abort1.notified() => {
                        let output = ProcessItem::Output("-LOGCLOSED-".into());
                        let value  = serde_json::to_vec(&output).unwrap();
                        let mut file = OpenOptions::new()
                            .append(true)
                            .open(&log_file_path)
                            .await
                            .unwrap();


                        file.write_all(&value).await.unwrap();
                        file.write_all(b"\n").await.ok();
                        break;
                    },
                    result = rx.recv() => match result {
                        None => break,
                        Some(output) => {

                            let mut file = OpenOptions::new()
                                .append(true)
                                .open(&log_file_path)
                                .await
                                .unwrap();


                            if let Ok(value) = serde_json::to_vec(&output) {
                                file.write_all(&value).await.unwrap();
                                file.write_all(b"\n").await.ok();
                            };
                        }
                    }

                }
            }
        });

        Ok(Self {
            purpose: purpose.as_ref().to_string(),
            file_path: new_log_file_path.as_ref().to_path_buf(),
            tx,
            abort,
            handle,
        })
    }

    pub fn add_process(&mut self, mut process: Box<dyn ProcessExt + Send>) -> Result<()> {
        let mut stream = process.spawn_and_stream()?;
        let cancel = self.abort.clone();
        let abort = process.aborter().unwrap();
        let tx = self.tx.clone();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancel.notified() => {
                        abort.notify_one();
                        break;
                    },
                    result = stream.next() => match result {
                        None => break,
                        Some(output) => {
                            if let Err(e) = tx.send(output) {
                                log::error!("Fail to send to channel {e}");
                            };
                        }
                    }

                }
            }
        });
        Ok(())
    }

    pub fn log<S: AsRef<str>>(&self, msg: S) -> Result<()> {
        self.tx.send(ProcessItem::Output(msg.as_ref().into())).ok();
        Ok(())
    }

    pub fn error<S: AsRef<str>>(&self, msg: S) -> Result<()> {
        self.tx.send(ProcessItem::Error(msg.as_ref().into())).ok();
        Ok(())
    }

    /// Get a reference to the logger's abort.
    #[must_use]
    pub fn get_aborter(&self) -> Arc<Notify> {
        self.abort.clone()
    }

    /// Explicitly Abort/Consume logger
    pub async fn abort(self) -> Result<()> {
        self.abort.notify_waiters();
        self.handle.await.map_err(Into::into)
    }

    /// Get logging task to send to client
    pub fn to_logging_task(&self) -> LoggingTask {
        LoggingTask {
            path: self.file_path.clone(),
            status: xbase_proto::LoggingTaskStatus::Consuming,
            purpose: self.purpose.clone(),
        }
    }
}

#[cfg(test)]
async fn stream_log_file<P: AsRef<Path>>(path: P) -> Result<ProcessStream> {
    let mut lines = linemux::MuxedLines::new()?;
    lines.add_file(path.as_ref()).await?;
    Ok(stream! {
        while let Ok(Some(item)) = lines.next_line().await {
            let line = item.line();
            if let Ok(value) = serde_json::from_str::<ProcessItem>(&line){
                if value == ProcessItem::Output("-LOGCLOSED-".into()) {
                    break;
                }
                yield value
            } else {
                log::error!("Fail {line}");

            }
        }
    }
    .boxed())
}

#[tokio::test]
async fn test_logger() -> Result<()> {
    log::setup("/tmp", "testsock", log::Level::DEBUG, true)?;

    let address = "/private/tmp/test_process.log";
    let mut logger = Logger::new(&address, "Testing").await?;
    let mut stream = stream_log_file(&address).await?;

    log::info!("Adding a process");
    logger.add_process(Box::new(
        xclog::XCLogger::new(
            "/Users/tami5/repos/swift/yabaimaster",
            &[
                "clean",
                "build",
                "-configuration",
                "Debug",
                "-target",
                "YabaiMaster",
            ],
        )
        .unwrap(),
    ))?;
    logger.add_process(Box::new(
        xclog::XCLogger::new(
            "/Users/tami5/repos/swift/wordle/",
            &[
                "clean",
                "build",
                "-configuration",
                "Debug",
                "-target",
                "Wordle",
            ],
        )
        .unwrap(),
    ))?;

    // let abort = logger.get_aborter();
    // tokio::spawn(async move {
    //     tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    //     abort.notify_waiters();
    // });

    tokio::spawn(async move {
        while let Some(output) = stream.next().await {
            log::info!("{output}");
        }
    });
    logger.handle.await.unwrap();

    Ok(())
}
