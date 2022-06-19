#![allow(dead_code)]

use crate::watch::InternalState;
use notify::{Event as NotifyEvent, EventKind as NotifyEventKind};
use std::{
    fmt,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use wax::Any;

pub enum EventKind {
    None,
    FileCreated,
    FolderCreated,
    FolderRemoved,
    FileUpdated,
    FileRenamed,
    FileRemoved,
    Other(NotifyEventKind),
}

impl Default for EventKind {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Default)]
pub struct Event {
    path: PathBuf,
    file_name: String,
    kind: EventKind,
    last_path: Arc<Mutex<PathBuf>>,
}

impl Event {
    pub fn new<'a>(
        ignore: &'a Any<'a>,
        state: &InternalState,
        mut event: NotifyEvent,
    ) -> Option<Self> {
        use notify::event::{CreateKind, DataChange, ModifyKind, RemoveKind};
        use NotifyEventKind::*;

        if event.paths.len() > 1 {
            log::error!("[FsEvent] More than one path! {:#?}", event)
        }

        let kind = match event.kind {
            Create(CreateKind::File) => EventKind::FileCreated,
            Create(CreateKind::Folder) => EventKind::FolderCreated,
            Modify(ModifyKind::Data(DataChange::Content)) => EventKind::FileUpdated,
            Modify(ModifyKind::Name(_)) => EventKind::FileRenamed,
            Remove(RemoveKind::File) => EventKind::FileRemoved,
            Remove(RemoveKind::Folder) => EventKind::FolderRemoved,
            kind => EventKind::Other(kind),
        };

        let path = event.paths.swap_remove(0);
        let file_name = match path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => {
                log::error!("[FsEvent] Unable to get event file path name!!! {path:?}",);
                Default::default()
            }
        };

        let is_match = wax::Pattern::is_match;

        // Skip ignore paths
        if is_match(ignore, &*path.to_string_lossy()) {
            log::trace!(r#"[WatchService] Ignored "{file_name}""#);
            return None;
        }

        // Skip if Unsupported event
        if let EventKind::Other(kind) = kind {
            log::trace!(r#"[WatchService] Skip {:?} of "{file_name}""#, kind,);
            return None;
        }

        let event = Self {
            path,
            file_name,
            kind,
            last_path: state.last_path(),
        };

        // Skip when last run was less then 1 second agot
        let last_run = state.last_run();
        if !(last_run > 1) {
            log::trace!("[WatchService] Skip [last_run: {last_run}] [{event}]");
            return None;
        }

        Some(event)
    }

    /// Returns `true` if the watch event kind is [`EventKind::FileUpdated`]
    pub fn is_content_update_event(&self) -> bool {
        matches!(self.kind, EventKind::FileUpdated)
    }

    /// Returns `true` if the watch event kind is [`EventKind::FileCreated`] or
    /// [`EventKind::FolderCretaed`].
    pub fn is_create_event(&self) -> bool {
        matches!(self.kind, EventKind::FileCreated) || matches!(self.kind, EventKind::FolderCreated)
    }

    /// Returns `true` if the watch event kind is [`EventKind::FileRemoved`] or
    /// [`EventKind::FolderRemoved`].
    pub fn is_remove_event(&self) -> bool {
        matches!(self.kind, EventKind::FileRemoved) || matches!(self.kind, EventKind::FolderCreated)
    }

    /// Returns `true` if the watch event kind is [`EventKind::FileRenamed`].
    pub fn is_rename_event(&self) -> bool {
        matches!(self.kind, EventKind::FileRenamed)
    }

    /// Returns `true` if the watch event kind is [`EventKind::Other`].
    #[must_use]
    pub fn is_other_event(&self) -> bool {
        matches!(self.kind, EventKind::Other(..))
    }

    /// Get a reference to the event's kind.
    #[must_use]
    pub fn kind(&self) -> &EventKind {
        &self.kind
    }

    /// Get a mutable reference to the event's file name.
    #[must_use]
    pub fn file_name(&self) -> &String {
        &self.file_name
    }

    /// Get a reference to the event's path.
    #[must_use]
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Get the event's is seen.
    #[must_use]
    pub fn is_seen(&self) -> bool {
        log::trace!("{}", self.file_name);

        if self.file_name.eq("project.yml") {
            return false;
        }
        let mut last_path = match self.last_path.lock() {
            Ok(path) => path,
            Err(err) => {
                log::error!("{err}");
                err.into_inner()
            }
        };

        if last_path.eq(self.path()) {
            true
        } else {
            *last_path = self.path.clone();
            false
        }
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use EventKind::*;
        let event_name = match &self.kind {
            FolderCreated | FileCreated => "Created",
            FolderRemoved | FileRemoved => "Removed",
            FileUpdated => "Updated",
            FileRenamed => "Renamed",
            Other(event) => {
                log::trace!("[FsEvent] Other: {:?}", event);
                "other"
            }
            _ => "",
        };
        write!(f, "{:?} [{event_name}]", self.file_name)
    }
}
