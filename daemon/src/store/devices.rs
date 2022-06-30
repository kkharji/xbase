use crate::device::Device;
use serde::Serialize;
use simctl::Simctl;
use std::collections::HashMap;
use xbase_proto::DeviceLookup;

#[derive(Debug, Serialize, derive_deref_rs::Deref)]
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
        if let Some(ref udid) = lookup.id {
            self.get(udid).cloned()
        } else {
            None
        }
    }
}
