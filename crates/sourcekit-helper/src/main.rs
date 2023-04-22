use anyhow::{anyhow, Context, Result};
use bsp_server::types::{
    BuildTargetSources, BuildTargetSourcesResult, InitializeBuild, Url, WorkspaceBuildTargetsResult,
};
use bsp_server::{Connection, Message, Request, RequestId, Response};
use once_cell::sync::OnceCell;
use serde_json::{json, Value};
use std::path::Path;
use std::sync::Mutex;
use std::time::SystemTime;
use std::{collections::HashMap, path::PathBuf};
use tracing::Level;
use xclog::{XCCompilationDatabase, XCCompileArgs, XCCompileCommand};

mod extensions;
mod helpers;
mod tracing_setup;

use extensions::*;
use helpers::*;

static SERVER_NAME: &str = "Xbase";
static SERVER_VERSION: &str = "0.3";
static STATE: OnceCell<Mutex<State>> = OnceCell::new();

type Conn = Connection;
type Id = RequestId;

#[derive(Debug)]
pub struct State {
    compile_db: XCCompilationDatabase,
    file_args: HashMap<PathBuf, XCCompileArgs>,
    root_path: PathBuf,
    compile_filepath: PathBuf,
    last_modified: SystemTime,
}

fn state() -> &'static Mutex<State> {
    &STATE.get().unwrap()
}

/// Initialize Server
fn initialize(params: &InitializeBuild) -> Result<InitializeBuild> {
    let root_path = params.root_path();
    let config_filepath = root_path.join("buildServer.json");
    let root_uri = params.root_uri();
    let compile_filepath = get_compile_filepath(root_uri).unwrap();
    let cache_path = get_build_cache_dir(&root_path)?;
    let index_store_path = get_index_store_path(&cache_path, &config_filepath);
    let compile_db = XCCompilationDatabase::try_from_filepath(&compile_filepath)?;

    let attr = std::fs::metadata(&compile_filepath)?;
    let last_modified = attr.modified()?;

    let response = InitializeBuild::new(
        SERVER_NAME,
        SERVER_VERSION,
        params.version(),
        params.root_uri().clone(),
        params.capabilities().clone(),
        json!({
            "indexDatabasePath": format!("{cache_path}/indexDatabasePath"),
            "indexStorePath": index_store_path,
        }),
    );
    tracing::trace!("{response:#?}");

    STATE
        .set(Mutex::new(State {
            root_path,
            file_args: Default::default(),
            compile_filepath,
            compile_db,
            last_modified,
        }))
        .unwrap();
    Ok(response)
}

fn get_compile_args<'a>(path: impl AsRef<Path>) -> Result<XCCompileArgs> {
    let mut state = state().lock().unwrap();
    let path = path.as_ref();
    let file_name = path.file_name().and_then(|v| v.to_str()).unwrap();

    if state.last_modified != std::fs::metadata(&state.compile_filepath)?.modified()? {
        state.compile_db = XCCompilationDatabase::try_from_filepath(&state.compile_filepath)?;
        state.file_args = Default::default();
    }

    if state.file_args.contains_key(path) {
        tracing::debug!("[{file_name}] Using Cached file args");
        state.file_args.get(path)
    } else {
        tracing::debug!("[{file_name}] Querying compile_db");
        let file_args = state
            .compile_db
            .iter()
            .flat_map(XCCompileCommand::compile_flags)
            .flatten()
            .collect::<HashMap<_, _>>();

        state.file_args.extend(file_args);
        state.file_args.get(path)
    }
    .map(|r| r.clone())
    .ok_or_else(|| anyhow!("Missing compile arguments for {path:?}"))
}

/// Register or unregister a file options for changes. On change, must send
/// SourceKitOptionsChanged with list of compiler options to compile the
/// file.
// #[tracing::instrument(name = "RegisterForChanges", skip_all)]
fn register_for_changes(conn: &Conn, id: Id, params: OptionsChangedRequest) -> Result<()> {
    // Empty response, ensure response before notification
    conn.send(Response::ok(id, Value::Null))?;

    if !matches!(params.action, RegisterAction::Register) {
        tracing::error!("Unhandled params: {:?}", params);
        return Ok(());
    }

    let filepath = params
        .uri
        .to_file_path()
        .map_err(|_| anyhow!("Invalid File URI: {:?}", params.uri))?;
    // tracing::info!("{filepath}");
    let root_path = state().lock().unwrap().root_path.clone();
    let uri = Url::from_directory_path(root_path).ok();
    let args = get_compile_args(filepath)?.to_vec();

    let notification: Message =
        OptionsChangedNotification::new(params.uri, args, uri).try_into()?;
    tracing::info!("");

    conn.send(notification)?;

    Ok(())
}

/// List of compiler options necessary to compile a file.
// #[tracing::instrument(name = "SourceKitOptions", skip_all)]
fn sourcekit_options(conn: &Conn, id: Id, params: OptionsRequest) -> Result<()> {
    let filepath = params.uri.path();
    tracing::info!("{filepath}");

    let root_path = state().lock().unwrap().root_path.clone();
    let uri = Url::from_directory_path(root_path).ok();
    let args = get_compile_args(filepath)?.to_vec();
    let response = OptionsResponse::new(args, uri).as_response(id);

    conn.send(response)?;

    Ok(())
}

/// Process Workspace BuildTarget request
// #[tracing::instrument(name = "WorkspaceBuildTargets", skip_all)]
fn workspace_build_targets(conn: &Conn, id: Id) -> Result<()> {
    tracing::debug!("Processing");
    let response = WorkspaceBuildTargetsResult::new(vec![]);

    conn.send((id, response))?;

    Ok(())
}

/// Process BuildTarget output paths
// #[tracing::instrument(name = "BuildTargetsOutputPaths", skip_all)]
fn output_paths(conn: &Conn, id: Id, params: BuildTargetOutputPathsRequest) -> Result<()> {
    tracing::debug!("Processing {params:#?}");
    let response = BuildTargetOutputPathsResponse::new(vec![]).as_response(id);

    conn.send(response)?;

    Ok(())
}

/// Process BuildTarget Sources Request
// #[tracing::instrument(name = "BuildTargetsSources", skip_all)]
fn build_target_sources(conn: &Conn, id: Id, params: BuildTargetSources) -> Result<()> {
    tracing::debug!("Processing {params:#?}");
    let response = BuildTargetSourcesResult::new(vec![]);
    conn.send((id, response))?;
    Ok(())
}

/// Return Default response for unhandled requests.
fn default_response(conn: &Conn, id: &Id, method: &str, params: Value) -> Result<()> {
    tracing::warn!("Unable to handle:\n\n{:#?}\n", method);
    tracing::debug!("Got Params:\n\n{:#?}\n", params);
    conn.send(Response::err(
        id.clone(),
        123,
        format!("unhandled method {method}"),
    ))?;
    Ok(())
}

/// Handle Shutdown Request
fn handle_shutdown(conn: &Conn, req: &Request) -> Result<bool> {
    if conn.handle_shutdown(&req).context("Shutdown server")? {
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Handle incoming messages
fn handle_message(conn: &Conn, msg: Message) -> Result<()> {
    match msg {
        Message::Request(req) => {
            match req {
                Request::WorkspaceBuildTargets(id) => {
                    // WorkspaceBuildTargets
                    workspace_build_targets(&conn, id)
                }
                Request::BuildTargetSources(id, value) => {
                    // BuildTargetSources
                    build_target_sources(conn, id, value)
                }
                Request::Custom(id, method, params) => match method {
                    OptionsChangedRequest::METHOD => {
                        // OptionsChangedRequest
                        register_for_changes(&conn, id, params.try_into()?)
                    }
                    OptionsRequest::METHOD => {
                        // OptionsRequest
                        sourcekit_options(&conn, id, params.try_into()?)
                    }
                    BuildTargetOutputPathsRequest::METHOD => {
                        // BuildTargetOutputPathsRequest
                        output_paths(&conn, id, params.try_into()?)
                    }
                    method => default_response(&conn, &id, method, params),
                },
                _ => {
                    let (id, method, params) = (req.id(), req.method(), req.params()?);
                    default_response(&conn, id, method, params)
                }
            }
        }
        Message::Response(_) => {
            tracing::warn!("skipping \n\n{:?}\n", msg);
            Ok(())
        }
        Message::Notification(_) => {
            tracing::warn!("skipping \n\n{:?}\n", msg);
            Ok(())
        }
    }
}

fn main() -> Result<()> {
    tracing_setup::setup("/tmp/", "xbase-build-server.log", Level::DEBUG, false)?;

    let (conn, io_threads) = Connection::stdio();
    tracing::info!("Started");
    conn.initialize(|params| match initialize(&params) {
        Err(err) => {
            tracing::error!("Failed to Initialize: ${}", err);
            panic!("Initialization Failure: ${}", err);
        }
        Ok(build) => build,
    })?;
    tracing::info!("Initialized");

    for msg in &conn.receiver {
        if let Message::Request(ref req) = msg {
            match handle_shutdown(&conn, req) {
                Err(err) => tracing::error!("Failure to shutdown server {:#?}", err),
                Ok(should_break) => {
                    if should_break {
                        tracing::info!("Shutdown");
                        break;
                    }
                }
            };
        }

        if let Err(err) = handle_message(&conn, msg) {
            tracing::error!("{:?}", err);
        }
    }

    io_threads.join()?;
    // tracing::info!("Ended");
    Ok(())
}
