use bsp_server::{Connection, Message, Request};
use tracing::Level;
use xbase::Result;
use xbase::{
    server::{BuildServer, BuildTargetOutputPathsRequest, OptionsChangedRequest, OptionsRequest},
    util::tracing::install_tracing,
};

fn main() -> Result<()> {
    install_tracing("/tmp/", "xbase-server.log", Level::DEBUG, false)?;
    let (conn, io_threads) = Connection::stdio();
    let mut server = BuildServer::default();

    tracing::info!("Started");

    conn.initialize(|params| {
        let res = BuildServer::new(&params).expect("Initialize");
        server = res.1;
        res.0
    })?;

    tracing::info!("Initialized");

    for msg in &conn.receiver {
        if let Message::Request(ref req) = msg {
            match server.handle_shutdown(&conn, req) {
                Err(err) => tracing::error!("Failure to shutdown server {:#?}", err),
                Ok(should_break) => {
                    if should_break {
                        tracing::info!("Shutdown");
                        break;
                    }
                }
            };
        }

        if let Err(err) = handle_message(&mut server, &conn, msg) {
            tracing::error!("{:?}", err);
        }
    }

    io_threads.join()?;
    tracing::info!("Ended");
    Ok(())
}

fn handle_message(server: &mut BuildServer, conn: &Connection, msg: Message) -> Result<()> {
    match msg {
        Message::Request(req) => {
            match req {
                Request::WorkspaceBuildTargets(id) => {
                    // WorkspaceBuildTargets
                    server.workspace_build_targets(&conn, id)
                }
                Request::BuildTargetSources(id, value) => {
                    // BuildTargetSources
                    server.build_target_sources(&conn, id, value)
                }
                Request::Custom(id, method, params) => match method {
                    OptionsChangedRequest::METHOD => {
                        // OptionsChangedRequest
                        server.register_for_changes(&conn, id, params.try_into()?)
                    }
                    OptionsRequest::METHOD => {
                        // OptionsRequest
                        server.sourcekit_options(&conn, id, params.try_into()?)
                    }
                    BuildTargetOutputPathsRequest::METHOD => {
                        // BuildTargetOutputPathsRequest
                        server.output_paths(&conn, id, params.try_into()?)
                    }
                    method => server.default_response(&conn, &id, method, params),
                },
                _ => {
                    let (id, method, params) = (req.id(), req.method(), req.params()?);
                    server.default_response(&conn, id, method, params)
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
