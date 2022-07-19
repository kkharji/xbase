mod message;
mod task;

pub use self::message::*;
pub use task::*;
use tracing::instrument;

use crate::util::extensions::PathExt;
use crate::Result;
use process_stream::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{mpsc::*, Mutex, Notify};
use tokio::task::JoinHandle;

/// Broadcast server to send task to clients
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
    tx: UnboundedSender<(Option<u32>, Message)>,
    /// Abort notifier to stop the logger
    abort: Arc<Notify>,
    /// Socket listeners
    #[allow(dead_code)]
    listeners: Arc<Mutex<HashMap<u32, UnixStream>>>,
}

impl Broadcast {
    pub const ROOT: &'static str = "/private/tmp/xbase";

    #[instrument(parent = None, name = "Broadcaster", skip_all, fields(name = root.as_ref().name().unwrap()))]
    pub async fn new(root: impl AsRef<Path>) -> Result<Self> {
        let (tx, rx) = unbounded_channel();
        let name = format!("{}.socket", root.as_ref().unique_name().unwrap());
        let base = PathBuf::from(Self::ROOT);

        if !base.exists() {
            tokio::fs::create_dir(Self::ROOT).await?;
        }

        let address = base.join(name);
        let name = root.as_ref().name().unwrap();

        if address.exists() {
            tokio::fs::remove_file(&address).await.ok();
        };

        let abort: Arc<Notify> = Default::default();
        let listeners: Arc<Mutex<HashMap<u32, UnixStream>>> = Default::default();

        let listener = UnixListener::bind(&address)?;
        let server = tokio::spawn(Self::start_server(
            name.clone(),
            listener,
            abort.clone(),
            listeners.clone(),
        ));
        let handle = tokio::spawn(Self::start_messages_handler(
            name,
            rx,
            abort.clone(),
            listeners.clone(),
        ));

        tracing::info!("Created");

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

    /// Start Broadcast server and start accepting clients
    #[instrument(parent = None, name = "Broadcaster", skip_all, fields(name=name))]
    async fn start_server(
        name: String,
        listener: UnixListener,
        abort: Arc<Notify>,
        listeners: Arc<Mutex<HashMap<u32, UnixStream>>>,
    ) {
        loop {
            tokio::select! {
                _ = abort.notified() => {
                    tracing::info!("[Dropped]");
                    break
                },
                Ok((mut stream, _)) = listener.accept() => {
                    let mut listeners = listeners.lock().await;

                    let (reader, _) = stream.split();

                    let mut buf = String::default();
                    let mut reader = BufReader::new(reader);

                    // let reader = BufReader::new(&stream);
                    match reader.read_line(&mut buf).await {
                        Ok(_) => match buf.trim().parse::<u32>() {
                            Ok(id) =>  {
                                tracing::info!("Connected [{id}]");
                                listeners.insert(id, stream);
                            }
                            Err(err) => {
                                tracing::error!("Failed to parse id as u32: {err}");

                            }
                        }
                        Err(err) => {
                            tracing::error!("Failed to connect a client: {err}");
                        }
                    }
                }
            }
        }
    }

    /// Start message handler
    /// This loop receive messages and write them on all connected clients.
    #[instrument(parent = None, name = "Broadcaster", skip_all, fields(name=name))]
    async fn start_messages_handler(
        name: String,
        mut rx: UnboundedReceiver<(Option<u32>, Message)>,
        abort: Arc<Notify>,
        listeners: Arc<Mutex<HashMap<u32, UnixStream>>>,
    ) {
        loop {
            tokio::select! {
                _ = abort.notified() => { break; },
                result = rx.recv() => match result {
                    None => break,
                    Some((id, message)) => {
                        let listeners =  listeners.clone();
                        let mut listeners = listeners.lock().await;
                        if let Message::Disconnect = message {
                            listeners.remove(&id.unwrap());
                            continue;
                        }

                        match serde_json::to_string(&message) {
                            Ok(mut value) => {
                                tracing::trace!("{value}");
                                value.push('\n');
                                if let Some(id) = id {
                                    if let Some(stream) = listeners.get_mut(&id) {
                                        stream.write_all(value.as_bytes()).await.ok();
                                        stream.flush().await.ok();
                                    } else {
                                        tracing::error!("[CLIENT WITH {id} NOT FOUND]")
                                    }
                                } else {
                                    for (_, listener) in listeners.iter_mut() {
                                        listener.write_all(value.as_bytes()).await.ok();
                                        listener.flush().await.ok();
                                    };
                                }
                            },
                            Err(err) => tracing::warn!("SendError: `{message:?}` = `{err}`"),
                        }
                    }
                }
            }
        }
    }

    pub fn send(&self, id: Option<u32>, message: Message) {
        self.tx.send((id, message)).ok();
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
    /// Tell connected clients to open logger
    pub fn open_logger(&self) {
        self.send(None, Message::OpenLogger)
    }

    /// Tell connected clients to reload language server
    pub fn reload_lsp_server(&self) {
        self.send(None, Message::ReloadLspServer)
    }

    pub fn update_current_task<S: AsRef<str>>(&self, content: S, level: ContentLevel) {
        self.send(
            None,
            Message::UpdateCurrentTask {
                content: content.as_ref().into(),
                level,
            },
        )
    }

    pub fn finish_current_task(&self, success: bool) {
        self.send(
            None,
            Message::FinishCurrentTask {
                status: if success {
                    TaskStatus::Succeeded
                } else {
                    TaskStatus::Failed
                },
            },
        )
    }

    pub fn set_state(&self, id: Option<u32>, state: State) {
        self.send(id, Message::SetState(state))
    }

    /// Notify clients with a message
    fn notify<S: AsRef<str>>(&self, msg: S, level: ContentLevel) {
        let msg = msg.as_ref();
        self.send(
            None,
            Message::Notify {
                content: msg.to_string(),
                level,
            },
        )
    }

    /// Log clients with a message
    fn log<S: AsRef<str>>(&self, msg: S, level: ContentLevel) {
        let msg = msg.as_ref();
        self.send(
            None,
            Message::Log {
                content: msg.to_string(),
                level,
            },
        )
    }

    /// Notify client with a message and id
    fn notify_with_id<S: AsRef<str>>(&self, msg: S, id: u32, level: ContentLevel) {
        let msg = msg.as_ref();
        self.send(
            Some(id),
            Message::Notify {
                content: msg.to_string(),
                level,
            },
        )
    }

    /// Log client with a message and id
    fn log_with_id<S: AsRef<str>>(&self, msg: S, id: u32, level: ContentLevel) {
        let msg = msg.as_ref();
        self.send(
            Some(id),
            Message::Log {
                content: msg.to_string(),
                level,
            },
        )
    }

    /// Notify clients with a message
    pub fn info<S: AsRef<str>>(&self, msg: S) {
        self.notify(msg, ContentLevel::Info)
    }

    /// Notify clients with an error message
    pub fn error<S: AsRef<str>>(&self, msg: S) {
        tracing::error!("{}", msg.as_ref());
        self.notify(msg, ContentLevel::Error)
    }

    /// Notify clients with a warn message
    pub fn warn<S: AsRef<str>>(&self, msg: S) {
        tracing::warn!("{}", msg.as_ref());
        self.notify(msg, ContentLevel::Warn)
    }

    /// Notify clients with a trace message
    pub fn trace<S: AsRef<str>>(&self, msg: S) {
        tracing::trace!("{}", msg.as_ref());
        self.notify(msg, ContentLevel::Trace)
    }

    /// Notify clients with a debug message
    pub fn debug<S: AsRef<str>>(&self, msg: S) {
        tracing::debug!("{}", msg.as_ref());
        self.notify(msg, ContentLevel::Debug)
    }

    /// Notify clients with a message
    pub fn log_info<S: AsRef<str>>(&self, msg: S) {
        self.log(msg, ContentLevel::Info)
    }

    /// Notify clients with an error message
    pub fn log_error<S: AsRef<str>>(&self, msg: S) {
        tracing::error!("{}", msg.as_ref());
        self.log(msg, ContentLevel::Error)
    }

    /// Notify clients with a warn message
    pub fn log_warn<S: AsRef<str>>(&self, msg: S) {
        tracing::warn!("{}", msg.as_ref());
        self.log(msg, ContentLevel::Warn)
    }

    /// Notify clients with a trace message
    pub fn log_trace<S: AsRef<str>>(&self, msg: S) {
        tracing::trace!("{}", msg.as_ref());
        self.log(msg, ContentLevel::Trace)
    }

    /// Notify clients with a debug message
    pub fn log_debug<S: AsRef<str>>(&self, msg: S) {
        tracing::debug!("{}", msg.as_ref());
        self.log(msg, ContentLevel::Debug)
    }

    /// Notify a specific client with a message
    pub fn info_with_id<S: AsRef<str>>(&self, id: u32, msg: S) {
        self.notify_with_id(msg, id, ContentLevel::Info)
    }

    /// Notify a specifc client with an error message
    pub fn error_with_id<S: AsRef<str>>(&self, id: u32, msg: S) {
        tracing::error!("{}", msg.as_ref());
        self.notify_with_id(msg, id, ContentLevel::Error)
    }

    /// Notify clients with a warn message
    pub fn warn_with_id<S: AsRef<str>>(&self, id: u32, msg: S) {
        tracing::warn!("{}", msg.as_ref());
        self.notify_with_id(msg, id, ContentLevel::Warn)
    }

    /// Notify clients with a trace message
    pub fn trace_with_id<S: AsRef<str>>(&self, id: u32, msg: S) {
        tracing::trace!("{}", msg.as_ref());
        self.notify_with_id(msg, id, ContentLevel::Trace)
    }

    /// Notify clients with a debug message
    pub fn debug_with_id<S: AsRef<str>>(&self, id: u32, msg: S) {
        tracing::debug!("{}", msg.as_ref());
        self.notify_with_id(msg, id, ContentLevel::Debug)
    }

    /// Notify clients with a message
    pub fn log_info_with_id<S: AsRef<str>>(&self, id: u32, msg: S) {
        self.log_with_id(msg, id, ContentLevel::Info)
    }

    /// Notify clients with an error message
    pub fn log_error_with_id<S: AsRef<str>>(&self, id: u32, msg: S) {
        tracing::error!("{}", msg.as_ref());
        self.log_with_id(msg, id, ContentLevel::Error)
    }

    /// Notify clients with a warn message
    pub fn log_warn_with_id<S: AsRef<str>>(&self, id: u32, msg: S) {
        tracing::warn!("{}", msg.as_ref());
        self.log_with_id(msg, id, ContentLevel::Warn)
    }

    /// Notify clients with a trace message
    pub fn log_trace_with_id<S: AsRef<str>>(&self, id: u32, msg: S) {
        tracing::trace!("{}", msg.as_ref());
        self.log_with_id(msg, id, ContentLevel::Trace)
    }

    /// Notify clients with a debug message
    pub fn log_debug_with_id<S: AsRef<str>>(&self, id: u32, msg: S) {
        tracing::debug!("{}", msg.as_ref());
        self.log_with_id(msg, id, ContentLevel::Debug)
    }
}
