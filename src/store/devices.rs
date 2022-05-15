use serde::{Deserialize, Serialize};
use simctl::{Device, Simctl};

#[derive(Debug, Serialize, Deserialize)]
pub struct SimDevices(Vec<Device>);

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
                .collect(),
        )
    }
}
