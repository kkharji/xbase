//! Module for communicating with SourceKit Build Server Protocol.
mod extensions;

use crate::{Error, Result};
use anyhow::Context;
use bsp_server::{types::*, Connection, Message, Request, RequestId, Response};
use serde_json::{json, Value};
use std::{fs::read_to_string, path::PathBuf};
use tap::Pipe;

pub use extensions::*;

use crate::util::fs::get_build_cache_dir;

static SERVER_NAME: &str = "Xbase";
static SERVER_VERSION: &str = "0.1";

/// SourceKit-lsp Build Server
///
/// Currently focused on supporting compilation and code navigation through providing file compiled
/// arguments
#[derive(Debug, Default)]
// TODO: Clear build server state when .compile get updated
//
// Keep track of compile_filepath last modified state, and run state.clear() when it get changed
pub struct BuildServer {
    compile_filepath: Option<PathBuf>,
    root_path: PathBuf,
}

impl BuildServer {
    /// Create a new instance of BuildServer
    pub fn new(params: &InitializeBuild) -> Result<(InitializeBuild, Self)> {
        let root_path = params.root_path();
        let config_filepath = root_path.join("buildServer.json");
        let compile_filepath = get_compile_filepath(params);
        let response = get_initialize_response(params, &root_path, &config_filepath)?;
        Ok((
            response,
            Self {
                root_path,
                compile_filepath,
            },
        ))
    }

    /// Register or unregister a file options for changes. On change, must send
    /// SourceKitOptionsChanged with list of compiler options to compile the
    /// file.
    #[tracing::instrument(name = "RegisterForChanges", skip_all)]
    pub fn register_for_changes(
        &mut self,
        conn: &Connection,
        id: RequestId,
        params: OptionsChangedRequest,
    ) -> Result<()> {
        // Empty response, ensure response before notification
        conn.send(Response::ok(id, Value::Null))?;
        if !matches!(params.action, RegisterAction::Register) {
            tracing::error!("Unhandled params: {:?}", params);
            return Ok(());
        }

        let filepath = params.uri.path();

        tracing::info!("{filepath:?}");

        let root = self.root_path.pipe_ref(Url::from_directory_path).ok();
        let flags = self.get_compile_args_for_filepath(filepath)?;

        let notification: Message =
            OptionsChangedNotification::new(params.uri, flags, root).try_into()?;

        conn.send(notification)?;
        Ok(())
    }

    /// List of compiler options necessary to compile a file.
    #[tracing::instrument(name = "SourceKitOptions", skip_all)]
    pub fn sourcekit_options(
        &mut self,
        conn: &Connection,
        id: RequestId,
        params: OptionsRequest,
    ) -> Result<()> {
        let filepath = params.uri.path();
        tracing::info!("{filepath}");

        let root = self.root_path.pipe_ref(Url::from_directory_path).ok();
        let flags = self.get_compile_args_for_filepath(filepath)?;
        let response = OptionsResponse::new(flags, root).as_response(id);

        conn.send(response)?;

        Ok(())
    }

    /// Process Workspace BuildTarget request
    #[tracing::instrument(name = "WorkspaceBuildTargets", skip_all)]
    pub fn workspace_build_targets(&mut self, conn: &Connection, id: RequestId) -> Result<()> {
        tracing::debug!("Processing");
        let response = WorkspaceBuildTargetsResult::new(vec![]);

        conn.send((id, response))?;
        Ok(())
    }

    /// Process BuildTarget output paths
    #[tracing::instrument(name = "BuildTargetsOutputPaths", skip_all)]
    pub fn output_paths(
        &mut self,
        conn: &Connection,
        id: RequestId,
        _params: BuildTargetOutputPathsRequest,
    ) -> Result<()> {
        tracing::debug!("Processing");
        let response = BuildTargetOutputPathsResponse::new(vec![]).as_response(id);
        conn.send(response)?;

        Ok(())
    }

    /// Process BuildTarget Sources Request
    #[tracing::instrument(name = "BuildTargetsSources", skip_all)]
    pub fn build_target_sources(
        &mut self,
        conn: &Connection,
        id: RequestId,
        _params: BuildTargetSources,
    ) -> Result<()> {
        tracing::debug!("Processing");
        let response = BuildTargetSourcesResult::new(vec![]);
        conn.send((id, response))?;
        Ok(())
    }

    /// Return Default response for unhandled requests.
    pub fn default_response(
        &self,
        conn: &Connection,
        id: &RequestId,
        method: &str,
        params: Value,
    ) -> Result<()> {
        tracing::warn!("Unable to handle:\n\n{:#?}\n", method);
        tracing::debug!("Params:\n\n{:#?}\n", params);
        conn.send(Response::err(
            id.clone(),
            123,
            format!("unhandled method {method}"),
        ))?;
        Ok(())
    }

    /// Handle Shutdown Request
    pub fn handle_shutdown(&self, conn: &Connection, req: &Request) -> Result<bool> {
        if conn.handle_shutdown(&req).context("Shutdown server")? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl BuildServer {
    fn get_compile_args_for_filepath(&mut self, uri: &str) -> Result<Vec<String>> {
        let build_target_path = uri.pipe(PathBuf::from);
        let compile_filepath = self.compile_filepath.as_ref();

        match crate::constants::SERVER_STATE.clone().lock() {
            Ok(mut state) => state
                .get_compile_args_for_filepath(&build_target_path, compile_filepath)?
                .to_vec()
                .pipe(Result::Ok),
            Err(err) => {
                let msg = format!("fail to get file flags {err}");
                tracing::error!("{}", msg);
                return Err(Error::Lock(msg));
            }
        }
    }
}

/// Return InitializeBuild response
fn get_initialize_response(
    params: &InitializeBuild,
    root_path: &PathBuf,
    config_filepath: &PathBuf,
) -> Result<InitializeBuild> {
    let cache_path = get_build_cache_dir(&root_path)?;
    let index_store_path = get_index_store_path(&cache_path, &config_filepath);
    let data = json!({
        "indexDatabasePath": format!("{cache_path}/indexDatabasePath"),
        "indexStorePath": index_store_path,
    });

    InitializeBuild::new(
        SERVER_NAME,
        SERVER_VERSION,
        params.version(),
        params.root_uri().clone(),
        params.capabilities().clone(),
        data,
    )
    .pipe(Result::Ok)
}

/// Try to get indexStorePath from config_filepath or default to "{cache_path}/indexStorePath"
fn get_index_store_path(cache_path: &String, config_filepath: &PathBuf) -> String {
    let mut index_store_path = format!("{cache_path}/indexStorePath");
    if config_filepath.is_file() {
        if let Ok(content) = read_to_string(config_filepath) {
            if let Ok(config) = serde_json::from_str::<Value>(&content) {
                if let Some(Value::String(p)) = config.get("indexStorePath") {
                    index_store_path = p.clone()
                }
            };
        };
    }
    index_store_path
}

/// Try to get .compile filepath
fn get_compile_filepath(params: &InitializeBuild) -> Option<PathBuf> {
    params
        .root_uri()
        .join(".compile")
        .ok()?
        .path()
        .pipe(PathBuf::from)
        .pipe(|path| path.is_file().then(|| path))
}
