use futures::{
    Sink, SinkExt, Stream, StreamExt,
    channel::mpsc::{Receiver, Sender, channel},
};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::{FramedRead, FramedWrite};
use tower_lsp_server::jsonrpc::{self, Id, Request, Response};

use crate::codec::{LanguageServerCodec, ParseError};

#[derive(Deserialize, Serialize)]
#[cfg_attr(test, derive(Debug, PartialEq))]
#[serde(untagged)]
pub enum Message {
    /// A response message.
    Response(Response),
    /// A request or notification message.
    Request(Request),
}

pub trait LanguageServerHandle
where
    Self: Send + Sync + 'static,
{
    type Requests: Sink<Request> + Unpin + 'static;
    type Responses: Stream<Item = Message> + Send;

    fn split(self) -> (Self::Requests, Self::Responses);
}

pub struct MpscHandle {
    tx: Sender<Request>,
    rx: Receiver<Message>,
}

impl MpscHandle {
    pub fn from_stdio<I: AsyncRead + Send + Unpin + 'static, O: AsyncWrite + Send + 'static>(
        buffer_size: usize,
        input: I,
        output: O,
    ) -> Self {
        let (request_tx, request_rx) = channel::<Request>(0);
        let (mut response_tx, response_rx) = channel(0);

        let mut framed_stdin = FramedRead::new(input, LanguageServerCodec::<Message>::default());
        let framed_stdout = FramedWrite::new(output, LanguageServerCodec::default());

        let forward_requests = request_rx
            .map(async |request| Ok(request))
            .buffer_unordered(buffer_size)
            .forward(framed_stdout);

        tokio::spawn(async move {
            while let Some(message) = framed_stdin.next().await {
                match message {
                    Ok(message) => {
                        let _ = response_tx.send(message).await;
                    }
                    Err(err) => {
                        let message = Message::Response(Response::from_error(
                            Id::Null,
                            to_jsonrpc_error(err),
                        ));
                        let _ = response_tx.send(message).await;
                    }
                }
            }
        });

        tokio::spawn(forward_requests);

        Self {
            tx: request_tx,
            rx: response_rx,
        }
    }
}

fn to_jsonrpc_error(err: ParseError) -> jsonrpc::Error {
    match err {
        ParseError::Body(err) if err.is_data() => jsonrpc::Error::invalid_request(),
        _ => jsonrpc::Error::parse_error(),
    }
}

impl LanguageServerHandle for MpscHandle {
    type Requests = Sender<Request>;
    type Responses = Receiver<Message>;

    fn split(self) -> (Self::Requests, Self::Responses) {
        let Self { tx, rx } = self;
        (tx, rx)
    }
}
