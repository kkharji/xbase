use crate::logger::Logger;
use crate::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Weak};
use xbase_proto::IntoResult;

#[derive(Default, Debug, derive_deref_rs::Deref)]
pub struct LoggerStore(HashMap<PathBuf, Arc<Logger>>);

// TODO(projects): presist a list of projects paths and information
impl LoggerStore {
    pub fn push(&mut self, logger: Logger) -> Weak<Logger> {
        let key = logger.log_path().to_path_buf();
        log::trace!("Logger added");
        let strong = Arc::new(logger);
        let weak = Arc::downgrade(&strong);

        self.0.insert(key, strong);

        weak
    }

    pub fn get_by_project_root(&self, project_root: &PathBuf) -> Vec<&Arc<Logger>> {
        self.0
            .iter()
            .filter(|(_, l)| l.project_root() == project_root)
            .map(|(_, l)| l)
            .collect::<Vec<_>>()
    }

    pub fn get<P>(&self, key: P) -> Result<Arc<Logger>>
    where
        P: AsRef<Path> + std::fmt::Debug,
    {
        self.0
            .get(key.as_ref())
            .into_result("Logger", key)
            .map(Clone::clone)
    }

    pub fn remove_all_by_project_root(&mut self, project_root: &PathBuf) {
        self.0.retain(|_, l| {
            if l.project_root() != project_root {
                true
            } else {
                l.abort();

                false
            }
        });
    }

    pub fn remove<P: AsRef<Path>>(&mut self, key: P) {
        self.0.remove(key.as_ref()).map(|l| {
            l.abort();
        });
    }
}
