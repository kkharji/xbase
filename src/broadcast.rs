mod message;
mod task;

pub use self::message::*;
pub use task::*;

use crate::util::extensions::PathExt;
use crate::Result;
use process_stream::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tap::Pipe;
use tokio::io::AsyncWriteExt;
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
        let name = address.file_name().unwrap().to_str().unwrap();

        if address.exists() {
            tracing::warn!("[{address:?}] Exists, removing ...");
            tokio::fs::remove_file(&address).await.ok();
        };

        let abort: Arc<Notify> = Default::default();
        let listeners: Arc<Mutex<Vec<UnixStream>>> = Default::default();

        tracing::info!("[{name}] Initialized");
        let server = Self::start_server(address.clone(), abort.clone(), listeners.clone())?;
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

    /// Start Broadcast server and start accepting clients
    fn start_server(
        address: PathBuf,
        abort: Arc<Notify>,
        listeners: Arc<Mutex<Vec<UnixStream>>>,
    ) -> Result<JoinHandle<()>> {
        let listener = UnixListener::bind(&address)?;
        tokio::spawn(async move {
            let name = address.file_name().unwrap().to_str().unwrap();
            loop {
                let listeners = listeners.clone();
                tokio::select! {
                    _ = abort.notified() => {
                        tracing::info!("[{name}] Closed");
                        tokio::fs::remove_file(&address).await.ok();
                        break
                    },
                    Ok((stream, _)) = listener.accept() => {

                        let mut listeners = listeners.lock().await;
                        listeners.push(stream);
                        tracing::info!("[{name}] Registered new client");
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
                    _ = abort.notified() => { break; },
                    result = rx.recv() => match result {
                        None => break,
                        Some(output) => {
                            let listeners =  listeners.clone();
                            let mut listeners = listeners.lock().await;
                            match serde_json::to_string(&output) {
                                Ok(mut value) => {
                                    tracing::debug!("Sent: {value}");
                                    value.push('\n');
                                    for listener in listeners.iter_mut() {
                                        listener.write_all(value.as_bytes()).await.ok();
                                        listener.flush().await.ok();
                                    };
                                },
                                Err(err) => tracing::warn!("SendError: `{output:?}` = `{err}`"),
                            }
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
