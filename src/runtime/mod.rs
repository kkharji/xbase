mod message;
pub use message::*;

use crate::{server::*, *};
use std::{path::PathBuf, sync::Arc};
use tokio::sync::{mpsc, Notify};
use tracing::{info, instrument};

/// ProjectRuntime
pub struct ProjectRuntime {
    /// Project Name
    name: String,
    /// Project Data
    project: ProjectImpl,
    /// Client Broadcaster
    broadcaster: Arc<Broadcast>,
    /// Receiver to receive PRMessages,
    receiver: mpsc::UnboundedReceiver<PRMessage>,
    /// Sender to send PRMessage&s,
    sender: mpsc::UnboundedSender<PRMessage>,
    /// Build/Run requests subscribed to changes
    watcher_subscribers: WatchSubscribers,
    /// Build/Run requests subscribed to changes
    watcher_state: WatcherState,
    /// Connect clients id
    clients: u32,
    /// Notifer to notify listeners that this runtime is no longer active
    abort: Arc<Notify>,
}

impl ProjectRuntime {
    #[instrument(parent = None, name = "Runtime", skip_all, fields(name = root.name().unwrap()))]
    pub async fn new(root: PathBuf) -> Result<(Self, PRMessageSender)> {
        info!("[Initializing] ------------------------");
        let (sender, receiver) = mpsc::unbounded_channel::<PRMessage>();
        let broadcaster = Arc::new(Broadcast::new(&root).await?);
        let project = project::project(&root, &broadcaster).await?;
        let rsender = PRMessageSender::new(&root, broadcaster.address(), &sender);
        let name = project.name().to_string();
        let watcher_subscribers = WatchSubscribers::new(&name);
        let runtime = Self {
            name,
            clients: Default::default(),
            abort: Default::default(),
            watcher_state: WatcherState::new(),
            watcher_subscribers,
            broadcaster,
            project,
            receiver,
            sender,
        };
        Ok((runtime, rsender))
    }

    /// Start Runtime Loop
    #[instrument(parent = None, name = "Runtime", skip_all, fields(name = self.name))]
    pub async fn start(mut self, id: u32) {
        tokio::spawn(
            Watcher::new(
                &self.name,
                &self.watcher_state,
                &self.sender,
                &self.abort,
                self.project.root(),
                self.project.watchignore(),
            )
            .start(),
        );

        self.on_connect(id);

        info!("[Initialized] -------------------------");
        while let Some(message) = self.receiver.recv().await {
            match message {
                PRMessage::Connect(id) => self.on_connect(id),
                PRMessage::Disconnect(id) => {
                    info!("Disconnected [{id}]");
                    self.clients -= 1;
                    self.broadcaster.send(Some(id), Message::Disconnect);
                    if self.clients.eq(&0) {
                        self.broadcaster.abort();
                        self.abort.notify_waiters();
                        tokio::spawn(async move { runtimes().await.remove(self.project.root()) });
                        break;
                    }
                }
                PRMessage::FSEvent(event) => self.on_fs_event(event).await,
                PRMessage::Run(req) => self.on_run(req).await,
                PRMessage::Build(req) => self.on_build(req).await,
            }
        }
        info!("[Dropped]");
    }

    fn on_connect(&mut self, id: u32) {
        info!("Connected [{id}]");
        self.clients += 1;
        let msg = format!("[{}] Registered", self.name);
        self.broadcaster.info_with_id(id, msg);
        self.set_client_project_state(Some(id));
        self.set_client_runner_state(id);
    }

    #[instrument(parent = None, name = "FSWatcher", skip_all, fields(name = self.name))]
    async fn on_fs_event(&mut self, event: Event) {
        let name = &self.name;

        info!("Processing {event}");
        if event.is_create_event()
            || event.is_remove_event()
            || event.is_content_update_event()
            || event.is_rename_event() && !event.is_seen()
        {
            let ensure_setup = self.project.ensure_setup(Some(&event), &self.broadcaster);
            match ensure_setup.await {
                Err(e) => self.broadcaster.error(format!("[{name}] {e}")),
                Ok(true) => self.set_client_project_state(None),
                _ => {}
            };
        }

        self.watcher_subscribers
            .trigger(&mut self.project, &event, &self.broadcaster)
            .await;

        info!("Processed {event}");

        self.watcher_state.update_debounce();
    }

    #[instrument(parent = None, name = "FSWatcher", skip_all, fields(name = self.name))]
    async fn on_run(&mut self, req: RunRequest) {
        info!("Running {}", req.settings.target);
        let is_watch = if !req.operation.is_stop() {
            req.operation.is_watch()
        } else {
            self.watcher_subscribers.remove(&req).await;
            return;
        };
        let service = req.into_service();
        let event = Event::default();
        let res = service.trigger(&mut self.project, &event, &self.broadcaster);
        if let Err(err) = res.await {
            let msg = format!("[{}] failed to start runner {err}", self.name);
            self.broadcaster.error(msg);
        }
        info!("Ran {}", service.settings.target);
        if is_watch {
            self.watcher_subscribers.add(service);
        }
    }

    #[instrument(parent = None, name = "FSWatcher", skip_all, fields(name = self.name))]
    async fn on_build(&mut self, req: BuildRequest) {
        let is_watch = if !req.operation.is_stop() {
            req.operation.is_watch()
        } else {
            self.watcher_subscribers.remove(&req).await;
            return;
        };

        info!("Building {}", req.settings.target);
        let event = Event::default();
        let res = req.trigger(&mut self.project, &event, &self.broadcaster);
        if let Err(err) = res.await {
            let msg = format!("[{}] failed to start runner {err}", self.name);
            self.broadcaster.error(msg);
        }
        info!("Built {}", req.settings.target);
        if is_watch {
            self.watcher_subscribers.add(req);
        }
    }

    fn set_client_project_state(&mut self, id: Option<u32>) {
        let info = ProjectInfo {
            watchlist: self.watcher_subscribers.keys(),
            targets: self.project.targets().clone(),
        };
        self.broadcaster.set_state(id, State::ProjectInfo(info))
    }

    fn set_client_runner_state(&mut self, id: u32) {
        self.broadcaster
            .set_state(Some(id), State::Runners(Runners::default()));
    }
}
