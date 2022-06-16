use serde::{Deserialize, Serialize};
use xbase_proto::DeviceLookup;
use {crate::types::Device, simctl::Simctl, std::collections::HashMap};

#[derive(Debug, Serialize, Deserialize, derive_deref_rs::Deref)]
pub struct Devices(#[deref] HashMap<String, Device>);

impl Default for Devices {
    fn default() -> Self {
        Self(
            Simctl::new()
                .list()
                .unwrap()
                .devices()
                .to_vec()
                .into_iter()
                .filter(|d| d.is_available)
                .map(|d| (d.udid.clone(), Device::from(d)))
                .collect(),
        )
    }
}

impl Devices {
    /// Get Device from Device lookup
    pub fn from_lookup(&self, lookup: DeviceLookup) -> Option<Device> {
        if let Some(ref udid) = lookup.udid {
            self.get(udid).cloned()
        } else {
            None
        }
    }
}
