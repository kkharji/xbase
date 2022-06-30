#![allow(unused_imports, unused_macros)]
use crate::Result;
use process_stream::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tap::Pipe;
use tokio::io::AsyncWriteExt;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{mpsc::*, Mutex, Notify};
use tokio::task::JoinHandle;
use xbase_proto::{Message, PathExt, StatuslineState, Task};

/// Boradcast server to send task to clients
#[derive(Debug)]
pub struct Broadcast {
    /// Project root for which the logger is created for.
    root: PathBuf,
    /// Logger path
    address: PathBuf,
    /// Logger handler
    pub handle: JoinHandle<()>,
    /// Server handler
    pub server: JoinHandle<()>,
    /// Sender to be used within the server to write items to file_path
    tx: UnboundedSender<Message>,
    /// Abort notifier to stop the logger
    abort: Arc<Notify>,
    /// Socket listeners
    #[allow(dead_code)]
    listeners: Arc<Mutex<Vec<UnixStream>>>,
}

impl Broadcast {
    pub const ROOT: &'static str = "/private/tmp/xbase";

    pub async fn new(root: impl AsRef<Path>) -> Result<Self> {
        let (tx, rx) = unbounded_channel();
        let name = format!("{}.socket", root.as_ref().unique_name().unwrap());
        let base = PathBuf::from(Self::ROOT);

        if !base.exists() {
            tokio::fs::create_dir(Self::ROOT).await?;
        }

        let address = base.join(name);

        if address.exists() {
            log::warn!("address {address:?} should have been removed automatically, removing ..");
            tokio::fs::remove_file(&address).await.ok();
        };

        let abort: Arc<Notify> = Default::default();
        let listeners: Arc<Mutex<Vec<UnixStream>>> = Default::default();

        let server = Self::start_server(&address, abort.clone(), listeners.clone())?;
        let handle = Self::start_messages_handler(rx, abort.clone(), listeners.clone())?;

        Ok(Self {
            root: root.as_ref().to_path_buf(),
            tx,
            abort,
            handle,
            listeners,
            server,
            address,
        })
    }

    /// Set the process stderr/stdout to be consumed and transformed to messages to be boradcasted
    /// as logs
    ///
    /// Return receiver for single message, whether the process successes or failed
    pub fn consume(&self, mut process: Box<dyn ProcessExt + Send>) -> Result<Receiver<bool>> {
        let mut stream = process.spawn_and_stream()?;
        let cancel = self.abort.clone();
        let abort = process.aborter().unwrap();
        let tx = self.tx.clone();
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
                        None => break,
                        Some(output) => {
                            if let Some(succ) = output.is_success() {
                                send_status.send(succ).await.ok();
                            }
                            if let Err(e) = tx.send(output.into()) {
                                log::error!("Fail to send to channel {e}");
                            };
                        }
                    }

                };
            }
        });
        Ok(recv_status)
    }

    /// Start Boradcast server and start accepting clients
    fn start_server(
        address: &PathBuf,
        abort: Arc<Notify>,
        listeners: Arc<Mutex<Vec<UnixStream>>>,
    ) -> Result<JoinHandle<()>> {
        let listener = UnixListener::bind(&address)?;
        tokio::spawn(async move {
            loop {
                let listeners = listeners.clone();
                tokio::select! {
                    _ = abort.notified() => {
                        log::info!("Closing server");
                        break
                    },
                    Ok((stream, _)) = listener.accept() => {

                        let mut listeners = listeners.lock().await;
                        listeners.push(stream);
                        log::info!("Connected");
                    }
                }
            }
        })
        .pipe(Ok)
    }

    /// Start message handler
    /// This loop receive messages and write them on all connected clients.
    fn start_messages_handler(
        mut rx: UnboundedReceiver<Message>,
        abort: Arc<Notify>,
        listeners: Arc<Mutex<Vec<UnixStream>>>,
    ) -> Result<JoinHandle<()>> {
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = abort.notified() => { log::info!("Stopping logging"); break; },
                    result = rx.recv() => match result {
                        None => break,
                        Some(output) => {
                            let mut listeners =  listeners.lock().await;
                            if let Ok(value) = serde_json::to_string(&output) {
                                for listener in listeners.iter_mut() {
                                    listener.write_all(format!("{value}\n").as_bytes()).await.ok();
                                    listener.flush().await.ok();
                                };
                            };

                        }
                    }
                }
            }
        })
        .pipe(Ok)
    }

    /// Explicitly Abort/Consume logger
    pub fn abort(&self) {
        self.abort.notify_waiters();
    }

    /// Get a reference to the logger's project root.
    #[must_use]
    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    /// Get a reference to the logger's log path.
    #[must_use]
    pub fn address(&self) -> &PathBuf {
        &self.address
    }
}

impl Broadcast {
    pub fn log_step<S: AsRef<str>>(&self, msg: S) {
        log::info!("{}", msg.as_ref());
        let sep = ".".repeat(73);
        self.tx.send(Message::log_info(msg)).ok();
        log::info!("{sep}");
        self.tx.send(Message::log_info(&sep)).ok();
    }

    pub fn log_separator(&self) {
        let sep = ".".repeat(73);
        log::info!("{sep}");
        self.tx.send(Message::log_info(&sep)).ok();
    }

    pub fn info<S: AsRef<str>>(&self, msg: S) {
        log::info!("{}", msg.as_ref());
        self.tx.send(msg.as_ref().into()).ok();
    }

    pub fn error<S: AsRef<str>>(&self, msg: S) {
        log::error!("{}", msg.as_ref());
        self.tx.send(Message::notify_error(msg)).ok();
    }

    pub fn warn<S: AsRef<str>>(&self, msg: S) {
        log::warn!("{}", msg.as_ref());
        self.tx.send(Message::notify_warn(msg)).ok();
    }

    pub fn trace<S: AsRef<str>>(&self, msg: S) {
        log::trace!("{}", msg.as_ref());
        self.tx.send(Message::notify_trace(msg)).ok();
    }

    pub fn debug<S: AsRef<str>>(&self, msg: S) {
        log::debug!("{}", msg.as_ref());
        self.tx.send(Message::notify_debug(msg)).ok();
    }

    pub fn log_info<S: AsRef<str>>(&self, msg: S) {
        self.tx.send(Message::log_info(msg)).ok();
    }

    pub fn log_error<S: AsRef<str>>(&self, msg: S) {
        log::error!("{}", msg.as_ref());
        self.tx.send(Message::log_error(msg)).ok();
    }

    pub fn log_warn<S: AsRef<str>>(&self, msg: S) {
        log::warn!("{}", msg.as_ref());
        self.tx.send(Message::log_warn(msg)).ok();
    }

    pub fn log_trace<S: AsRef<str>>(&self, msg: S) {
        log::trace!("{}", msg.as_ref());
        self.tx.send(Message::log_trace(msg)).ok();
    }

    pub fn log_debug<S: AsRef<str>>(&self, msg: S) {
        log::debug!("{}", msg.as_ref());
        self.tx.send(Message::log_debug(msg)).ok();
    }

    pub fn update_statusline(&self, state: StatuslineState) {
        self.tx
            .send(Message::Execute(Task::UpdateStatusline(state)))
            .ok();
    }
}
