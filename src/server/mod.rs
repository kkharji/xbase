mod build;
mod drop;
mod register;
mod request;
mod response;
mod run;

use std::os::unix::prelude::AsRawFd;
use tokio::net::unix::{ReadHalf, WriteHalf};
use tokio_serde::{formats::*, SymmetricallyFramed};
use tokio_util::codec::{BytesCodec, FramedRead, FramedWrite};
use tracing::instrument;
use typescript_type_def::TypeDef;

pub use {build::*, drop::*, register::*, request::*, response::*, run::*};

/// Stream of Requests to read Requests from
struct RequestStream;

/// Stream of Responses to write response to
struct ResponseStream;

/// Trait that must be implemented by All Request members
#[async_trait::async_trait]
pub trait RequestHandler<T: serde::Serialize> {
    async fn handle(self) -> crate::Result<T>;
}

/// Generic over T and Framed Write/Read
type F<T, F> = SymmetricallyFramed<F, T, SymmetricalJson<T>>;

impl RequestStream {
    fn new<'a>(r: ReadHalf<'a>) -> F<Request, FramedRead<ReadHalf<'a>, BytesCodec>> {
        let transport = FramedRead::new(r, BytesCodec::default());
        F::new(transport, SymmetricalJson::default())
    }
}

impl ResponseStream {
    fn new<'a>(w: WriteHalf<'a>) -> F<Response, FramedWrite<WriteHalf<'a>, BytesCodec>> {
        let transport = FramedWrite::new(w, BytesCodec::default());
        F::new(transport, SymmetricalJson::default())
    }
}

/// Future that await and process client requests.
#[instrument(parent = None, name = "Client", skip_all, fields(fd = stream.as_raw_fd()))]
pub async fn handle(mut stream: tokio::net::UnixStream) {
    use futures::{SinkExt, TryStreamExt};
    use tracing::{error, info};
    info!("Connected");

    // Client Registered roots
    let mut roots = vec![];
    let (reader, writer) = stream.split();
    let (mut reader, mut writer) = (RequestStream::new(reader), ResponseStream::new(writer));
    let mut id = 0;

    loop {
        match reader.try_next().await {
            Ok(Some(request)) => {
                if let Request::Register(r) = &request {
                    id = r.id;
                    roots.push(r.root.clone())
                };
                let response = request.handle().await;
                let send_res = writer.send(response).await;
                send_res.map_err(|err| error!("Send Error: {err}")).ok();
            }
            Err(err) => error!("Read Error: {err:#?}"),
            Ok(None) => {
                Request::Drop(DropRequest { id, roots }).handle().await;
                break;
            }
        }
    }
    info!("Disconnected");
}
