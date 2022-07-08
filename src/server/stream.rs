use super::*;
use tap::Pipe;
use tokio::net::unix::{ReadHalf, WriteHalf};
use tokio::net::UnixStream;
use tokio_serde::{formats::*, SymmetricallyFramed};
use tokio_util::codec::{BytesCodec, FramedRead, FramedWrite};

/// Generic over T and Framed Write/Read
type FrameTransport<T, F> = SymmetricallyFramed<F, T, SymmetricalJson<T>>;

/// Stream of Requests to read Requests from
pub type RequestStream<'a> = FrameTransport<Request, FramedRead<ReadHalf<'a>, BytesCodec>>;

/// Stream of Responses to write response to
pub type ResponseStream<'a> = FrameTransport<Response, FramedWrite<WriteHalf<'a>, BytesCodec>>;

// Split UnixStream into (RequestStream, ResponseStream)
pub fn split(stream: &mut UnixStream) -> (RequestStream, ResponseStream) {
    stream.split().pipe(|(r, w)| {
        // Stream to read Requests from stream.
        let reader = RequestStream::new(
            FramedRead::new(r, BytesCodec::default()),
            SymmetricalJson::default(),
        );

        // Stream to write Response to stream.
        let writer = ResponseStream::new(
            FramedWrite::new(w, BytesCodec::default()),
            SymmetricalJson::default(),
        );

        (reader, writer)
    })
}
