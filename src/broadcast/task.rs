use super::*;

#[derive(Debug, Clone)]
pub struct Task {
    #[allow(dead_code)]
    task: TaskKind,
    #[allow(dead_code)]
    target: String,
    inner: Arc<Broadcast>,
}

impl Task {
    /// Create a new task with it's kind, the target, and broadcast to send message through
    pub fn new(task: TaskKind, target: &str, broadcast: Arc<Broadcast>) -> Task {
        broadcast
            .tx
            .send(Message::SetCurrentTask {
                kind: task.clone(),
                target: target.into(),
                status: TaskStatus::Processing,
            })
            .ok();
        Task {
            task,
            target: target.into(),
            inner: broadcast,
        }
    }

    fn update<S: AsRef<str>>(&self, level: ContentLevel, content: S) {
        let content = content.as_ref().into();
        let message = Message::UpdateCurrentTask { content, level };
        self.inner.tx.send(message).ok();
    }

    /// Update CurrentTask with info and content
    pub fn info<S: AsRef<str>>(&self, content: S) {
        self.update(ContentLevel::Info, content);
    }

    /// Update CurrentTask with debug and content
    pub fn debug<S: AsRef<str>>(&self, content: S) {
        self.update(ContentLevel::Debug, content);
    }
    /// Update CurrentTask with debug and content
    pub fn warn<S: AsRef<str>>(&self, content: S) {
        self.update(ContentLevel::Warn, content);
    }

    /// Update CurrentTask with trace and content
    pub fn trace<S: AsRef<str>>(&self, content: S) {
        self.update(ContentLevel::Trace, content);
    }

    /// Update CurrentTask with error and content
    pub fn error<S: AsRef<str>>(&self, content: S) {
        self.update(ContentLevel::Error, content);
    }

    /// Finish task with whether it was successfull or not
    pub fn finish(&self, success: bool) {
        self.inner
            .tx
            .send(Message::FinishCurrentTask {
                status: if success {
                    TaskStatus::Succeeded
                } else {
                    TaskStatus::Failed
                },
            })
            .ok();

        if !success {
            self.inner.open_logger();
        }
    }

    pub fn consume(&self, mut process: Box<dyn ProcessExt + Send>) -> Result<Receiver<bool>> {
        let mut stream = process.spawn_and_stream()?;
        let cancel = self.inner.abort.clone();
        let abort = process.aborter().unwrap();
        let this = self.clone();
        let (send_status, recv_status) = channel(1);

        tokio::spawn(async move {
            loop {
                let send_status = send_status.clone();
                tokio::select! {
                    _ = cancel.notified() => {
                        abort.notify_one();
                        send_status.send(false).await.unwrap_or_default();
                        break;
                    },
                    result = stream.next() => match result {
                        Some(output) => {
                            if let Some(succ) = output.is_success() {
                                this.finish(succ);
                                send_status.send(succ).await.ok();
                                break;
                            } else if let ProcessItem::Error(content) = output {
                                this.error(content)
                            } else if let ProcessItem::Output(content) = output {
                                if content.to_lowercase().contains("error") {
                                    this.error(content)
                                } else if content.to_lowercase().contains("warn") {
                                    this.warn(content)
                                } else {
                                    if content != "Resolving Packages" {
                                        this.info(content)
                                    };
                                }
                            }
                        }
                        None => break,
                    }
                };
            }
        });
        Ok(recv_status)
    }

    pub fn inner(&self) -> &Broadcast {
        self.inner.as_ref()
    }
}
