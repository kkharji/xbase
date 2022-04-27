use anyhow::Result;
use bsp_server::{Connection, Message, Request};
use tracing::Level;
use xcodebase::{
    server::{BuildServer, BuildTargetOutputPathsRequest, OptionsChangedRequest, OptionsRequest},
    util::tracing::install_tracing,
};

fn main() -> Result<()> {
    install_tracing("/tmp/", "xcodebase-server.log", Level::DEBUG, false)?;
    let (conn, io_threads) = Connection::stdio();
    let mut server = BuildServer::default();

    tracing::info!("Started");

    conn.initialize(|params| {
        let res = BuildServer::new(&params).expect("Initialize");
        tracing::info!("Initialized");
        server = res.1;
        res.0
    })?;

    for msg in &conn.receiver {
        match msg {
            Message::Request(req) => match req {
                Request::WorkspaceBuildTargets(id) => {
                    server.workspace_build_targets(&conn, id)?;
                }
                Request::BuildTargetSources(id, value) => {
                    server.build_target_sources(&conn, id, value)?;
                }
                Request::Custom(id, method, params) => match method {
                    OptionsChangedRequest::METHOD => {
                        server.register_for_changes(&conn, id, params.try_into()?)?;
                    }
                    OptionsRequest::METHOD => {
                        server.sourcekit_options(&conn, id, params.try_into()?)?;
                    }
                    BuildTargetOutputPathsRequest::METHOD => {
                        server.output_paths(&conn, id, params.try_into()?)?;
                    }
                    method => {
                        server.default_response(&conn, &id, method, params)?;
                    }
                },
                Request::Shutdown(_) => {
                    server.handle_shutdown(&conn, &req)?;
                    break;
                }
                _ => {
                    let (id, method, params) = (req.id(), req.method(), req.params()?);
                    server.default_response(&conn, id, method, params)?;
                }
            },

            Message::Response(_) => {
                tracing::warn!("skipping \n\n{:?}\n", msg);
            }
            Message::Notification(_) => {
                tracing::warn!("skipping \n\n{:?}\n", msg);
            }
        }
    }

    io_threads.join()?;
    tracing::info!("Ended");
    Ok(())
}
