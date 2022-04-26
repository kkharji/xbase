use bsp_server::{types::BuildTargetIdentifier, Message, Notification, RequestId, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tap::Pipe;
use url::Url;

/// The register for changes request is sent from the language
/// server to the build server to register or unregister for
/// changes in file options or dependencies. On changes a
/// FileOptionsChangedNotification is sent.
#[derive(Debug, Deserialize, Serialize)]
pub struct RegisterForChanges {
    /// The URI of the document to get options for.
    pub uri: Url,
    /// Whether to register or unregister for the file.
    pub action: RegisterAction,
}

impl RegisterForChanges {
    pub const METHOD: &'static str = "textDocument/registerForChanges";
    pub fn new(uri: Url, action: RegisterAction) -> Self {
        Self { uri, action }
    }
}

impl Into<RegisterForChanges> for Value {
    fn into(self) -> RegisterForChanges {
        serde_json::from_value(self).unwrap()
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RegisterAction {
    Register,
    Unregister,
}

/// The SourceKitOptions request is sent from the client to the server
/// to query for the list of compiler options necessary to compile this file.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceKitOptions {
    /// The URI of the document to get options for
    pub uri: Url,
}

impl SourceKitOptions {
    pub const METHOD: &'static str = "textDocument/sourceKitOptions";
}

impl Into<SourceKitOptions> for Value {
    fn into(self) -> SourceKitOptions {
        serde_json::from_value(self).unwrap()
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceKitOptionsResult {
    /// The compiler options required for the requested file.
    pub options: Vec<String>,
    /// The working directory for the compile command.
    pub working_directory: Option<Url>,
}

impl SourceKitOptionsResult {
    pub fn new(options: Vec<String>, working_directory: Option<Url>) -> Self {
        Self {
            options,
            working_directory,
        }
    }

    pub fn as_response(self, id: RequestId) -> Response {
        Response::ok(id, self)
    }
}

/// The SourceKitOptionsChanged is sent from the
/// build server to the language server when it detects
/// changes to a registered files build settings.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceKitOptionsChangedNotification {
    /// The URI of the document that has changed settings.
    pub uri: Url,
    /// The updated options for the registered file.
    pub updated_options: SourceKitOptionsResult,
}

impl SourceKitOptionsChangedNotification {
    pub fn new(uri: Url, options: Vec<String>, working_directory: Option<Url>) -> Self {
        SourceKitOptionsResult {
            options,
            working_directory,
        }
        .pipe(|updated_options| Self {
            uri,
            updated_options,
        })
    }
}

impl From<SourceKitOptionsChangedNotification> for Message {
    fn from(not: SourceKitOptionsChangedNotification) -> Message {
        Message::Notification(Notification::Custom(
            "build/sourceKitOptionsChanged",
            // WARN: Force Unwrap
            serde_json::to_value(not).unwrap(),
        ))
    }
}

/// The build target output paths request is sent from the client to the server
/// to query for the list of compilation output paths for a targets sources.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildTargetOutputPaths {
    pub targets: Vec<BuildTargetIdentifier>,
}

impl Into<BuildTargetOutputPaths> for Value {
    fn into(self) -> BuildTargetOutputPaths {
        serde_json::from_value(self).unwrap()
    }
}

impl BuildTargetOutputPaths {
    pub const METHOD: &'static str = "buildTarget/outputPaths";
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BuildTargetOutputPathsResult {
    pub items: Vec<OutputsItem>,
}

impl BuildTargetOutputPathsResult {
    pub fn new(items: Vec<OutputsItem>) -> Self {
        Self { items }
    }

    pub fn as_response(self, id: RequestId) -> Response {
        Response::ok(id, self)
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputsItem {
    pub target: BuildTargetIdentifier,
    /// The output paths for sources that belong to this build target.
    pub output_paths: Vec<Url>,
}

impl OutputsItem {
    pub fn new(target: BuildTargetIdentifier, output_paths: Vec<Url>) -> Self {
        Self {
            target,
            output_paths,
        }
    }
}
