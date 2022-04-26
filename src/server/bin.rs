use anyhow::Result;
use bsp_server::{Connection, Message, Request, Response};
use xcodebase::{
    install_tracing,
    server::{BuildServer, BuildTargetOutputPaths, RegisterForChanges, SourceKitOptions},
};

fn main() -> Result<()> {
    install_tracing(
        "/tmp/",
        "xcodebase-server.log",
        tracing::Level::DEBUG,
        false,
    )?;
    let (conn, io_threads) = Connection::stdio();

    tracing::info!("Started------------------------------");

    let mut server = BuildServer::default();
    conn.initialize(|params| {
        let res = BuildServer::new(&params).expect("Initialize");
        server = res.1;
        res.0
    })?;

    block(conn, server)?;

    io_threads.join()?;

    tracing::info!("Ended ------------------------------");

    Ok(())
}

fn block(conn: Connection, mut server: BuildServer) -> Result<()> {
    for msg in &conn.receiver {
        match msg {
            Message::Request(req) => {
                use Request::*;
                match req {
                    Shutdown(_) => {
                        conn.handle_shutdown(&req)?;
                        return Ok(());
                    }
                    WorkspaceBuildTargets(id) => server.build_targets(&conn, id)?,
                    BuildTargetSources(id, value) => server.sources(&conn, id, value)?,
                    Custom(id, RegisterForChanges::METHOD, params) => {
                        server.register_for_changes(&conn, id, params.into())?
                    }
                    Custom(id, SourceKitOptions::METHOD, params) => {
                        server.sourcekit_options(&conn, id, params.into())?
                    }
                    Custom(id, BuildTargetOutputPaths::METHOD, params) => {
                        server.output_paths(&conn, id, params.into())?
                    }
                    _ => {
                        let (id, method) = (req.id(), req.method());
                        tracing::warn!("Unable to handle:\n\n{:#?}\n", method);
                        conn.send(Response::err(
                            id.clone(),
                            123,
                            format!("unhandled method {method}"),
                        ))?;
                    }
                };
            }
            Message::Response(_) => {
                tracing::warn!("skipping \n\n{:?}\n", msg);
            }
            Message::Notification(_) => {
                tracing::warn!("skipping \n\n{:?}\n", msg);
            }
        }
    }
    Ok(())
}
