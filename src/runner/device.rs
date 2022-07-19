use derive_deref_rs::Deref;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, hash::Hash};
use tap::Pipe;
use typescript_type_def::TypeDef;
use xcodeproj::pbxproj::PBXTargetPlatform;

use crate::DeviceLookup;

#[derive(Clone, Debug, Serialize, derive_deref_rs::Deref)]
pub struct Device {
    pub platform: PBXTargetPlatform,
    #[serde(flatten)]
    #[deref]
    inner: simctl::Device,
}

#[derive(Debug, Serialize, Deref)]
pub struct Devices(HashMap<String, Device>);

impl std::fmt::Display for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
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

static DEVICES: Lazy<Devices> = Lazy::new(Default::default);

/// Represntaiton of Project runners index by Platfrom
#[derive(Debug, Serialize, Deserialize, TypeDef)]
pub struct Runners(HashMap<String, Vec<DeviceLookup>>);

impl Default for Runners {
    fn default() -> Self {
        let devices = &*DEVICES;
        vec![
            PBXTargetPlatform::IOS,
            PBXTargetPlatform::WatchOS,
            PBXTargetPlatform::TvOS,
        ]
        .into_iter()
        .map(|p| {
            (
                p.to_string(),
                devices
                    .iter()
                    .filter(|(_, d)| d.platform == p)
                    .map(|(id, d)| DeviceLookup::new(d.name.clone(), id.clone()))
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<HashMap<String, _>>()
        .pipe(Self)
    }
}

impl Default for Devices {
    fn default() -> Self {
        Devices(
            simctl::Simctl::new()
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
    pub fn from_lookup(lookup: Option<DeviceLookup>) -> Option<Device> {
        lookup.and_then(|d| DEVICES.get(&d.id)).cloned()
    }
}
