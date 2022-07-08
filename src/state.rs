use crate::runner::Devices;
use crate::*;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{Mutex, OwnedMutexGuard};

type ArcMutex<T> = Arc<Mutex<T>>;
type LazyArcMutex<T> = Lazy<Arc<Mutex<T>>>;

pub type Watchers = HashMap<PathBuf, ArcMutex<WatchService>>;
pub type Projects = HashMap<PathBuf, ArcMutex<ProjectImplementer>>;
pub type Broadcasters = HashMap<PathBuf, Arc<Broadcast>>;

static BROADCASTERS: LazyArcMutex<Broadcasters> = Lazy::new(Default::default);
static WATCHERS: LazyArcMutex<Watchers> = Lazy::new(Default::default);
static PROJECTS: LazyArcMutex<Projects> = Lazy::new(Default::default);
static DEVICES: Lazy<Devices> = Lazy::new(Default::default);

/// Get OwnedMutexGuard of Broadcasters
pub async fn broadcasters() -> OwnedMutexGuard<Broadcasters> {
    get_guard(&*BROADCASTERS).await
}

/// Get OwnedMutexGuard of daemon projects
pub async fn projects() -> OwnedMutexGuard<Projects> {
    get_guard(&*PROJECTS).await
}

/// Get OwnedMutexGuard of daemon projects watchers
pub async fn watchers() -> OwnedMutexGuard<Watchers> {
    get_guard(&*WATCHERS).await
}

/// Get static reference to registered simulator devices
pub fn devices() -> &'static Devices {
    &*DEVICES
}

/// Get daemon object primarly for client current root.
///
/// WARN: This shortcuts must be used carefully, or you will endup in dead lock
#[async_trait::async_trait]
pub trait TryGetDaemonObject {
    /// Try get mutex watcher using self
    async fn try_get_mutex_watcher(&self) -> Result<Arc<Mutex<WatchService>>>;

    /// Try get mutex project implementer using self
    async fn try_get_mutex_project(&self) -> Result<Arc<Mutex<ProjectImplementer>>>;

    /// Try get owned guard of a project implementer using self
    async fn try_get_project(&self) -> Result<OwnedMutexGuard<ProjectImplementer>> {
        Ok(self.try_get_mutex_project().await?.lock_owned().await)
    }

    /// Try get owned mutex guard of a Watcher using self
    async fn try_get_watcher(&self) -> Result<OwnedMutexGuard<WatchService>> {
        Ok(self.try_get_mutex_watcher().await?.lock_owned().await)
    }

    /// Try get owned mutex guard of a Watcher using self
    async fn try_get_broadcast(&self) -> Result<Arc<Broadcast>>;
}

#[async_trait::async_trait]
impl<S: AsRef<Path> + Send + Sync> TryGetDaemonObject for S {
    async fn try_get_mutex_watcher(&self) -> Result<Arc<Mutex<WatchService>>> {
        let root = self.as_ref();
        watchers()
            .await
            .get(root)
            .into_result("Watcher", root)
            .map(|v| v.clone())
    }

    /// Try get mutex project implementer using self
    async fn try_get_mutex_project(&self) -> Result<Arc<Mutex<ProjectImplementer>>> {
        let root = self.as_ref();
        projects()
            .await
            .get(root)
            .into_result("Project", root)
            .map(|v| v.clone())
    }

    /// Try get broadcaster using self
    async fn try_get_broadcast(&self) -> Result<Arc<Broadcast>> {
        let root = self.as_ref();
        broadcasters()
            .await
            .get(root)
            .into_result("Broadcaster", root)
            .map(|v| v.clone())
    }
}

async fn get_guard<T>(m: &'static Arc<Mutex<T>>) -> OwnedMutexGuard<T> {
    m.clone().lock_owned().await
}
