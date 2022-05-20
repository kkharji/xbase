use crate::types::SimDevice;
use serde::{Deserialize, Serialize};
use simctl::Simctl;
use std::collections::HashSet;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Serialize, Deserialize)]
pub struct SimDevices(HashSet<SimDevice>);

impl Deref for SimDevices {
    type Target = HashSet<SimDevice>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SimDevices {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for SimDevices {
    fn default() -> Self {
        Self(
            Simctl::new()
                .list()
                .unwrap()
                .devices()
                .to_vec()
                .into_iter()
                .filter(|d| d.is_available)
                .map(Into::into)
                .collect(),
        )
    }
}
