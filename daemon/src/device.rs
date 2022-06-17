use super::project::Platform;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

#[derive(Clone, Debug, Serialize, Deserialize, derive_deref_rs::Deref)]
pub struct Device {
    pub platform: Platform,
    #[serde(flatten)]
    #[deref]
    inner: simctl::Device,
}

impl Eq for Device {}

impl PartialEq for Device {
    fn eq(&self, other: &Self) -> bool {
        self.inner.udid == other.inner.udid
    }
}

impl Hash for Device {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.udid.hash(state)
    }
}

impl From<simctl::Device> for Device {
    fn from(inner: simctl::Device) -> Self {
        let ref id = inner.runtime_identifier;
        let platform = Platform::from_identifer(id);
        Self { inner, platform }
    }
}

impl Device {
    /// Get special build arguments to run on current device.
    // -sdk driverkit -sdk iphoneos -sdk macosx -sdk appletvos -sdk watchos
    pub fn special_build_args(&self) -> Vec<String> {
        match self.platform {
            Platform::IOS => vec!["-sdk".into(), "iphonesimulator".into()],
            Platform::WatchOS => vec!["-sdk".into(), "watchsimulator".into()],
            Platform::TvOS => vec!["-sdk".into(), "appletvsimulator".into()],
            Platform::MacOS => vec!["-sdk".into(), "macosx".into()],
            Platform::Unknown => vec![],
        }
    }
}
