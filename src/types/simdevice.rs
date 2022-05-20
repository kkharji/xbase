use serde::{Deserialize, Serialize};
use simctl::Device;
use std::hash::Hash;
use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SimDevice(Device);

impl Eq for SimDevice {}

impl PartialEq for SimDevice {
    fn eq(&self, other: &Self) -> bool {
        self.0.udid == other.0.udid
    }
}

impl Hash for SimDevice {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.udid.hash(state)
    }
}

impl Deref for SimDevice {
    type Target = Device;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SimDevice {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Device> for SimDevice {
    fn from(d: Device) -> Self {
        Self(d)
    }
}
