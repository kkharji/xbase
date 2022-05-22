use super::Platform;
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Device {
    pub platform: Platform,
    #[serde(flatten)]
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

impl Deref for Device {
    type Target = simctl::Device;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Device {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
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
