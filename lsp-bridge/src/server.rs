use crate::{
    codec::LanguageServerCodec,
    transport::{ExitedError, handle::Message},
};
use dashmap::{DashMap, Entry};
use futures::{
    FutureExt,
    channel::{
        mpsc::{Sender, UnboundedReceiver},
        oneshot,
    },
};
use futures::{Sink, SinkExt, StreamExt, channel::mpsc};
use serde_json::json;
use std::error::Error;
use std::fmt::{self, Formatter};
use std::{collections::VecDeque, sync::Arc};
use std::{
    fmt::Display,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    join,
};
use tokio_util::codec::{FramedRead, FramedWrite};
use tower::Service;
use tower_lsp_server::jsonrpc::{self, Id, Request, Response};

#[derive(Debug, Clone)]
pub struct Pending {
    pending: Arc<DashMap<Id, VecDeque<oneshot::Sender<Response>>>>,
    unmatched_responses: mpsc::UnboundedSender<Response>,
}

impl Pending {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<Response>) {
        let (response_tx, response_rx) = mpsc::unbounded();
        (
            Self {
                pending: Arc::default(),
                unmatched_responses: response_tx,
            },
            response_rx,
        )
    }
}

#[derive(Debug, Clone)]
pub enum SendError {
    Full,
    Disconnected,
    UnknownId(Id),
}

impl Display for SendError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "SendError: ")
    }
}

impl Error for SendError {}

impl From<futures::channel::mpsc::SendError> for SendError {
    fn from(value: futures::channel::mpsc::SendError) -> Self {
        if value.is_full() {
            SendError::Full
        } else {
            SendError::Disconnected
        }
    }
}

impl<T> From<mpsc::TrySendError<T>> for SendError {
    fn from(value: mpsc::TrySendError<T>) -> Self {
        if value.is_full() {
            SendError::Full
        } else {
            SendError::Disconnected
        }
    }
}

impl From<tower_lsp_server::ExitedError> for SendError {
    fn from(_: tower_lsp_server::ExitedError) -> Self {
        SendError::Disconnected
    }
}

impl Pending {
    pub fn insert(&self, id: Id) -> Option<oneshot::Receiver<Response>> {
        if id == Id::Null {
            return None;
        }

        let (response_tx, response_rx) = oneshot::channel();
        match self.pending.entry(id) {
            Entry::Occupied(mut o) => o.get_mut().push_back(response_tx),
            Entry::Vacant(v) => {
                v.insert(vec![response_tx].into());
            }
        };
        Some(response_rx)
    }
}

impl Sink<Response> for Pending {
    type Error = SendError;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(self: std::pin::Pin<&mut Self>, item: Response) -> Result<(), Self::Error> {
        if let Entry::Occupied(mut o) = self.pending.entry(item.id().clone()) {
            o.get_mut()
                .pop_front()
                .unwrap()
                .send(item)
                .map_err(|_| SendError::Disconnected)?;
            if o.get().is_empty() {
                o.remove();
            }
            Ok(())
        } else {
            self.unmatched_responses
                .unbounded_send(item)
                .map_err(|err| err.into())
        }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}

#[derive(Debug)]
pub struct ChildLanguageServer {
    pending: Pending,
    pub(crate) unmatched_responses_rx: UnboundedReceiver<Response>,
    pub(crate) pending_requests: UnboundedReceiver<Request>,
    pub(crate) output_tx: Sender<Message>,
}

impl ChildLanguageServer {
    pub fn connect<I, O>(input: I, output: O) -> Self
    where
        I: AsyncRead + Unpin + Send + 'static,
        O: AsyncWrite + Send + 'static,
    {
        let mut framed_input = FramedRead::new(input, LanguageServerCodec::<Message>::default());
        let (mut input_request_tx, input_request_rx) = mpsc::unbounded();
        let (pending, unmatched_responses_rx) = Pending::new();

        let mut _pending = pending.clone();
        let forward_input_messages = async move {
            while let Some(message) = framed_input.next().await {
                match message {
                    Ok(Message::Request(request)) => input_request_tx.send(request).await?,
                    Ok(Message::Response(response)) => _pending.send(response).await?,
                    Err(err) => {
                        _pending
                            .send(Response::from_error(
                                Id::Null,
                                jsonrpc::Error {
                                    code: jsonrpc::ErrorCode::ParseError,
                                    message: "Parse error occurred".into(),
                                    data: Some(json!({
                                        "error": &err.to_string(),
                                    })),
                                },
                            ))
                            .await?
                    }
                }
            }
            Ok::<_, SendError>(())
        };

        let framed_output = FramedWrite::new(output, LanguageServerCodec::default());
        let (output_message_tx, output_message_rx) = mpsc::channel(0);
        let forward_output_messages = output_message_rx.map(|m| Ok(m)).forward(framed_output);
        tokio::spawn(async move {
            join!(forward_input_messages, forward_output_messages)
        });

        Self {
            pending,
            unmatched_responses_rx,
            pending_requests: input_request_rx,
            output_tx: output_message_tx,
        }
    }

    pub async fn send_request(
        &mut self,
        request: Request,
    ) -> Result<Option<Response>, ExitedError> {
        let response_rx = request.id().and_then(|id| self.pending.insert(id.clone()));
        self.output_tx.send(Message::Request(request)).await?;
        if let Some(response_rx) = response_rx {
            let response = response_rx.await?;
            Ok(Some(response))
        } else {
            Ok(None)
        }
    }

    pub fn split(self) -> (LanguageServerSender, LanguageServerReceiver) {
        let Self {
            pending,
            unmatched_responses_rx,
            pending_requests,
            output_tx,
        } = self;
        (
            LanguageServerSender { pending, output_tx },
            LanguageServerReceiver {
                unmatched_responses_rx,
                pending_requests,
            },
        )
    }
}

impl Service<Request> for ChildLanguageServer {
    type Response = Option<Response>;
    type Error = ExitedError;
    type Future = CallLanguageServer;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        CallLanguageServer::new(req, self.pending.clone(), self.output_tx.clone())
    }
}

#[derive(Debug)]
pub struct CallLanguageServer {
    request: Option<Request>,
    pending: Pending,
    message_tx: Sender<Message>,
    send_done: bool,
    response_rx: Option<oneshot::Receiver<Response>>,
}

impl CallLanguageServer {
    pub fn new(request: Request, pending: Pending, message_tx: Sender<Message>) -> Self {
        CallLanguageServer {
            request: Some(request),
            pending,
            message_tx,
            send_done: false,
            response_rx: None,
        }
    }
}

impl Future for CallLanguageServer {
    type Output = Result<Option<Response>, ExitedError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(request) = self.request.take() {
            self.response_rx = request.id().cloned().and_then(|id| self.pending.insert(id));
            self.message_tx
                .start_send_unpin(Message::Request(request))?;
        }
        if self.send_done {
            if let Some(response_rx) = &mut self.response_rx {
                let poll = response_rx.poll_unpin(cx)?;
                if let Poll::Ready(response) = poll {
                    Poll::Ready(Ok(Some(response)))
                } else {
                    Poll::Pending
                }
            } else {
                Poll::Ready(Ok(None))
            }
        } else {
            let poll = self.message_tx.poll_flush_unpin(cx)?;
            if poll.is_pending() {
                Poll::Pending
            } else {
                self.send_done = true;
                self.poll(cx)
            }
        }
    }
}

impl From<mpsc::SendError> for ExitedError {
    fn from(_: mpsc::SendError) -> Self {
        ExitedError
    }
}

impl From<oneshot::Canceled> for ExitedError {
    fn from(_: oneshot::Canceled) -> Self {
        ExitedError
    }
}

#[derive(Clone)]
pub struct LanguageServerSender {
    pending: Pending,
    pub output_tx: Sender<Message>,
}

impl Service<Request> for LanguageServerSender {
    type Response = Option<Response>;
    type Error = ExitedError;
    type Future = CallLanguageServer;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        CallLanguageServer::new(req, self.pending.clone(), self.output_tx.clone())
    }
}

pub struct LanguageServerReceiver {
    unmatched_responses_rx: UnboundedReceiver<Response>,
    pending_requests: UnboundedReceiver<Request>,
}

impl LanguageServerReceiver {
    pub fn into_parts(self) -> (UnboundedReceiver<Response>, UnboundedReceiver<Request>) {
        let Self {
            unmatched_responses_rx,
            pending_requests,
        } = self;
        (unmatched_responses_rx, pending_requests)
    }
}
