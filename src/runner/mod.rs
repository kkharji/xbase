mod bin;
mod device;
mod simulator;

use crate::*;
use async_trait::async_trait;
use process_stream::{Process, ProcessExt, StreamExt};
use std::sync::Arc;
use std::sync::Weak;
use tokio::task::JoinHandle;

pub use {bin::*, device::*, simulator::*};

/// Run Service
pub struct RunService {
    pub key: String,
    pub root: PathBuf,
    pub handler: Arc<Mutex<Option<RunHandler>>>,
    pub settings: BuildSettings,
    pub device: Option<Device>,
}

impl RunService {
    pub fn new(
        device: Option<Device>,
        root: PathBuf,
        settings: BuildSettings,
        key: String,
    ) -> Self {
        Self {
            key,
            root,
            handler: Arc::new(Mutex::new(None)),
            settings,
            device,
        }
    }
}

impl std::fmt::Display for RunService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.key)
    }
}

#[async_trait::async_trait]
impl Watchable for RunService {
    async fn trigger(
        &self,
        project: &mut ProjectImpl,
        _event: &Event,
        broadcast: &Arc<Broadcast>,
    ) -> Result<()> {
        let Self { settings, .. } = self;

        let mut handler = self.handler.clone().lock_owned().await;

        handler.take().map(|v| {
            v.process().abort();
            v.inner().abort();
        });

        let device = self.device.as_ref();
        let target = &settings.target;
        let (runner, _args, mut recv) = project.get_runner(&settings, device, broadcast)?;

        if !recv.recv().await.unwrap_or_default() {
            return Err(crate::Error::Run(format!("{target} build failed")));
        }

        let task = Task::new(TaskKind::Run, target, broadcast.clone());

        let runner = runner.run(&task).await?;
        let broadcast = Arc::downgrade(broadcast);

        *handler = Some(RunHandler::new(target, runner, broadcast)?);

        Ok(())
    }

    /// A function that controls whether a a Watchable should restart
    async fn should_trigger(&self, event: &Event) -> bool {
        event.is_any_but_not_seen()
    }

    /// A function that controls whether a watchable should be droped
    async fn should_discard(&self, _event: &Event) -> bool {
        false
    }

    /// Drop watchable for watching a given file system
    async fn discard(&self) {
        self.handler.clone().lock_owned().await.take().map(|v| {
            v.process().abort();
            v.inner().abort();
        });
    }
}

/// Run Service Task Handler
pub struct RunHandler {
    process: Process,
    inner: JoinHandle<Result<()>>,
}

impl RunHandler {
    // Change the status of the process to running
    pub fn new(target: &String, mut process: Process, broadcast: Weak<Broadcast>) -> Result<Self> {
        let target = target.clone();
        let mut stream = process.spawn_and_stream()?;
        let abort = process.aborter().unwrap();

        let inner: _ = tokio::spawn(async move {
            // TODO: find a better way to close this!
            //
            // Right now it just wait till the user try print something
            while let Some(output) = stream.next().await {
                let ref mut broadcast = match broadcast.upgrade() {
                    Some(broadcast) => broadcast,
                    None => {
                        tracing::warn!("No client instance listening, closing runner ..");
                        abort.notify_waiters();
                        break;
                    }
                };

                use process_stream::ProcessItem::*;
                match output {
                    Output(msg) => {
                        if !msg.contains("ignoring singular matrix") {
                            broadcast.log_info(msg);
                        }
                    }
                    Error(msg) => {
                        broadcast.log_error(msg);
                    }
                    // TODO: this should be skipped when user re-run the app
                    Exit(code) => {
                        let success = &code == "0";
                        if success {
                            broadcast.log_info("Device Disconnected");
                        } else {
                            broadcast.log_error("Device Disconnected");
                        }
                        broadcast.finish_current_task(success);
                        tracing::info!("[{target}] Runner Closed");
                        break;
                    }
                };
            }

            drop(stream);

            Ok(())
        });

        Ok(Self { process, inner })
    }

    /// Get a reference to the run service handler's process.
    #[must_use]
    pub fn process(&self) -> &Process {
        &self.process
    }

    /// Get a reference to the run service handler's handler.
    #[must_use]
    pub fn inner(&self) -> &JoinHandle<Result<()>> {
        &self.inner
    }
}

#[async_trait]
pub trait Runner {
    async fn run<'a>(&self, task: &Task) -> Result<Process>;
}
