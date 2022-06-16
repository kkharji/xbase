use crate::types::*;
use crate::util::into_request;
use crate::util::value_or_default;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Build(BuildRequest),
    Run(RunRequest),
    Register(RegisterRequest),
    Drop(DropRequest),
}

into_request!(Build);
into_request!(Run);
into_request!(Register);
into_request!(Drop);

/// Request to build a particular project
#[derive(Debug, Serialize, Deserialize)]
pub struct BuildRequest {
    pub client: Client,
    pub settings: BuildSettings,
    #[serde(deserialize_with = "value_or_default")]
    pub direction: BufferDirection,
    #[serde(deserialize_with = "value_or_default")]
    pub ops: Operation,
}

/// Request to Run a particular project.
#[derive(Debug, Serialize, Deserialize)]
pub struct RunRequest {
    pub client: Client,
    pub settings: BuildSettings,
    #[serde(deserialize_with = "value_or_default")]
    pub device: DeviceLookup,
    #[serde(deserialize_with = "value_or_default")]
    pub direction: BufferDirection,
    #[serde(deserialize_with = "value_or_default")]
    pub ops: Operation,
}

/// Request to Register the given client.
#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub client: Client,
}

/// REquest to Drop the given client.
#[derive(Debug, Serialize, Deserialize)]
pub struct DropRequest {
    pub client: Client,
    #[serde(default)]
    pub remove_client: bool,
}
