use serde::{Deserialize, Serialize};
#[cfg(feature = "daemon")]
use {crate::types::Device, simctl::Simctl, std::collections::HashMap};

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "daemon", derive(derive_deref_rs::Deref))]
pub struct Devices(
    #[cfg(feature = "daemon")]
    #[cfg_attr(feature = "daemon", deref)]
    HashMap<String, Device>,
);

#[cfg(feature = "daemon")]
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

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct DeviceLookup {
    pub name: Option<String>,
    pub udid: Option<String>,
}

#[cfg(feature = "daemon")]
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
