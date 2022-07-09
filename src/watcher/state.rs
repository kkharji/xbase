use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::SystemTime,
};

pub struct InternalState {
    debounce: Arc<Mutex<SystemTime>>,
    last_path: Arc<Mutex<PathBuf>>,
}

impl Default for InternalState {
    fn default() -> Self {
        Self {
            debounce: Arc::new(Mutex::new(SystemTime::now())),
            last_path: Default::default(),
        }
    }
}

impl InternalState {
    pub fn update_debounce(&self) {
        let mut debounce = self.debounce.lock().unwrap();
        *debounce = SystemTime::now();
        tracing::trace!("Debounce updated!!!");
    }

    pub fn last_run(&self) -> u128 {
        self.debounce.lock().unwrap().elapsed().unwrap().as_millis()
    }

    /// Get a reference to the internal state's last path.
    #[must_use]
    pub fn last_path(&self) -> Arc<Mutex<PathBuf>> {
        self.last_path.clone()
    }
}
