use super::WatchService;
use serde::Serialize;
use std::fmt::Debug;

impl Serialize for WatchService {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_map(self.listeners.iter().map(|(key, _)| (key, true)))
    }
}

impl Debug for WatchService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let listners = self
            .listeners
            .iter()
            .map(|(key, _)| key.to_string())
            .collect::<Vec<String>>();

        f.debug_struct("WatchService")
            .field("listners", &listners)
            .field("handler", &self.handler)
            .finish()
    }
}
