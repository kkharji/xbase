use serde::Serialize;
use std::hash::Hash;
use xcodeproj::pbxproj::PBXTargetPlatform;

#[derive(Clone, Debug, Serialize, derive_deref_rs::Deref)]
pub struct Device {
    pub platform: PBXTargetPlatform,
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
        let platform = PBXTargetPlatform::from_identifer(id);
        Self { inner, platform }
    }
}

impl Device {
    /// Get special build arguments to run on current device.
    // -sdk driverkit -sdk iphoneos -sdk macosx -sdk appletvos -sdk watchos
    pub fn special_build_args(&self) -> Vec<String> {
        match self.platform {
            PBXTargetPlatform::IOS => vec!["-sdk".into(), "iphonesimulator".into()],
            PBXTargetPlatform::WatchOS => vec!["-sdk".into(), "watchsimulator".into()],
            PBXTargetPlatform::TvOS => vec!["-sdk".into(), "appletvsimulator".into()],
            PBXTargetPlatform::MacOS => vec!["-sdk".into(), "macosx".into()],
            PBXTargetPlatform::Unknown => vec![],
        }
    }
}
