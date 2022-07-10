use super::*;
use tokio::net::unix::{ReadHalf, WriteHalf};
use tokio_serde::{formats::*, SymmetricallyFramed};
use tokio_util::codec::{BytesCodec, FramedRead, FramedWrite};

/// Generic over T and Framed Write/Read
type F<T, F> = SymmetricallyFramed<F, T, SymmetricalJson<T>>;

/// Stream of Requests to read Requests from
struct RequestStream;

impl RequestStream {
    fn new<'a>(r: ReadHalf<'a>) -> F<Request, FramedRead<ReadHalf<'a>, BytesCodec>> {
        let transport = FramedRead::new(r, BytesCodec::default());
        F::new(transport, SymmetricalJson::default())
    }
}

/// Stream of Responses to write response to
struct ResponseStream;

impl ResponseStream {
    fn new<'a>(w: WriteHalf<'a>) -> F<Response, FramedWrite<WriteHalf<'a>, BytesCodec>> {
        let transport = FramedWrite::new(w, BytesCodec::default());
        F::new(transport, SymmetricalJson::default())
    }
}

/// Future that await and process client requests.
pub async fn handle(mut stream: tokio::net::UnixStream) {
    use futures::{FutureExt, SinkExt, TryStreamExt};
    use tracing::{error, info};

    info!("Handling a new client");

    let (reader, writer) = stream.split();
    let (mut reader, mut writer) = (RequestStream::new(reader), ResponseStream::new(writer));

    loop {
        match reader.try_next().await {
            Ok(Some(request)) => {
                let send_result = request.handle().then(|res| writer.send(res)).await;
                send_result.map_err(|err| error!("Send Error: {err}")).ok();
            }
            Err(err) => error!("Read Error: {err:#?}"),
            Ok(None) => break,
        }
    }
    info!("Disconnecting a client");
}