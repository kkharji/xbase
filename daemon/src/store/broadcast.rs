use crate::broadcast::Broadcast;
use crate::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Weak};
use xbase_proto::IntoResult;

#[derive(Default, Debug, derive_deref_rs::Deref)]
pub struct BroadcastStore(HashMap<PathBuf, Arc<Broadcast>>);

impl BroadcastStore {
    pub async fn get_or_init(&mut self, root: &PathBuf) -> Result<Weak<Broadcast>> {
        if let Ok(logger) = self.get(root) {
            Ok(Arc::downgrade(&logger))
        } else {
            log::trace!("Logger added");
            let logger = Broadcast::new(root).await?;
            let strong = Arc::new(logger);
            let weak = Arc::downgrade(&strong);

            self.0.insert(root.into(), strong);

            Ok(weak)
        }
    }

    pub fn get<P>(&self, key: P) -> Result<Arc<Broadcast>>
    where
        P: AsRef<Path> + std::fmt::Debug,
    {
        self.0
            .get(key.as_ref())
            .into_result("Logger", key)
            .map(Clone::clone)
    }

    pub fn remove<P: AsRef<Path>>(&mut self, key: P) {
        self.0.remove(key.as_ref()).map(|l| {
            l.abort();
        });
    }
}
