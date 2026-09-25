use std::{
    collections::VecDeque,
    error::Error,
    ffi::OsStr,
    fmt::{self, Display, Formatter},
    pin::Pin,
    process::Stdio,
    sync::Arc,
    task::{Context, Poll},
};

use dashmap::{DashMap, Entry};
use futures::{
    FutureExt, Sink, SinkExt, Stream, StreamExt,
    channel::{mpsc, oneshot},
    join,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    process::Command,
};
use tokio_util::codec::{FramedRead, FramedWrite};
use tower::Service;
use tower_lsp_server::jsonrpc::{self, Id, Request, Response};

use crate::codec::LanguageServerCodec;

#[derive(Deserialize, Serialize)]
#[cfg_attr(test, derive(Debug, PartialEq))]
#[serde(untagged)]
pub enum Message {
    /// A response message.
    Response(Response),
    /// A request or notification message.
    Request(Request),
}

pub trait LsHandle
where
    Self:
        Service<Request, Response = Option<Response>, Error = ExitedError> + Stream<Item = Message>,
{
    type LsService: Service<Request, Response = Option<Response>, Error = ExitedError>;
    type LsSocket: Stream<Item = Message>;

    fn from_stdio<I, O>(input: I, output: O) -> Self
    where
        I: AsyncRead + Unpin + Send + 'static,
        O: AsyncWrite + Send + 'static;

    fn split(self) -> (Self::LsService, Self::LsSocket);
}

pub trait LsHandleExt: LsHandle + Sized {
    fn spawn<S: AsRef<OsStr>, S2: AsRef<OsStr>, I: IntoIterator<Item = S2>>(
        command: S,
        args: I,
    ) -> Result<Self, std::io::Error> {
        let mut child = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        let input = child.stdout.take().unwrap();
        let output = child.stdin.take().unwrap();

        Ok(Self::from_stdio(input, output))
    }
}

impl<T: LsHandle> LsHandleExt for T {}

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

/// Error that occurs when attempting to call the language server after it has already exited.
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ExitedError;

impl std::error::Error for ExitedError {}

impl Display for ExitedError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("language server has exited")
    }
}

impl From<ExitedError> for jsonrpc::Error {
    fn from(_: ExitedError) -> Self {
        Self {
            code: jsonrpc::ErrorCode::ServerError(-333444),
            message: "language server has exited".into(),
            data: None,
        }
    }
}

impl From<SendError> for ExitedError {
    fn from(value: SendError) -> Self {
        match value {
            SendError::Disconnected => ExitedError,
            SendError::Full => unreachable!("Socket is full despite waiting until free"),
            SendError::UnknownId(_) => unreachable!(),
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

impl From<tower_lsp_server::ExitedError> for ExitedError {
    fn from(_: tower_lsp_server::ExitedError) -> Self {
        ExitedError
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
pub struct CallLanguageServer {
    request: Option<Request>,
    ls_sender: BaseLsService,
    send_done: bool,
    response_rx: Option<oneshot::Receiver<Response>>,
}

impl CallLanguageServer {
    pub fn new(request: Request, ls_sender: BaseLsService) -> Self {
        CallLanguageServer {
            request: Some(request),
            ls_sender,
            send_done: false,
            response_rx: None,
        }
    }
}

impl Future for CallLanguageServer {
    type Output = Result<Option<Response>, ExitedError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(request) = self.request.take() {
            self.response_rx = request
                .id()
                .cloned()
                .and_then(|id| self.ls_sender.pending.insert(id));
            self.ls_sender
                .output_tx
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
            let poll = self.ls_sender.output_tx.poll_flush_unpin(cx)?;
            if poll.is_pending() {
                Poll::Pending
            } else {
                self.send_done = true;
                self.poll(cx)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct BaseLsService {
    pending: Pending,
    pub output_tx: mpsc::Sender<Message>,
}

impl Service<Request> for BaseLsService {
    type Response = Option<Response>;
    type Error = ExitedError;
    type Future = CallLanguageServer;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.output_tx.poll_ready_unpin(cx).map_err(|e| e.into())
    }

    fn call(&mut self, req: Request) -> Self::Future {
        CallLanguageServer::new(req, self.clone())
    }
}

#[derive(Debug)]
pub struct BaseLsSocket {
    unmatched_responses_rx: mpsc::UnboundedReceiver<Response>,
    pending_requests: mpsc::UnboundedReceiver<Request>,

    /// This variable is necessary to toggle between polling the response stream or the request stream first, to ensure fairness
    was_previous_response: bool,
}

impl Stream for BaseLsSocket {
    type Item = Message;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.was_previous_response {
            match self.pending_requests.poll_next_unpin(cx) {
                Poll::Pending | Poll::Ready(None) => self
                    .unmatched_responses_rx
                    .poll_next_unpin(cx)
                    .map(|m| m.map(|m| Message::Response(m))),
                Poll::Ready(Some(request)) => {
                    self.was_previous_response = false;
                    Poll::Ready(Some(Message::Request(request)))
                }
            }
        } else {
            match self.unmatched_responses_rx.poll_next_unpin(cx) {
                Poll::Pending | Poll::Ready(None) => self
                    .pending_requests
                    .poll_next_unpin(cx)
                    .map(|m| m.map(|m| Message::Request(m))),
                Poll::Ready(Some(response)) => {
                    self.was_previous_response = true;
                    Poll::Ready(Some(Message::Response(response)))
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct BaseLsHandle {
    service: BaseLsService,
    socket: BaseLsSocket,
}

impl Service<Request> for BaseLsHandle {
    type Response = Option<Response>;
    type Error = ExitedError;
    type Future = CallLanguageServer;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        self.service.call(req)
    }
}

impl Stream for BaseLsHandle {
    type Item = Message;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.socket.poll_next_unpin(cx)
    }
}

impl LsHandle for BaseLsHandle {
    type LsService = BaseLsService;
    type LsSocket = BaseLsSocket;

    fn from_stdio<I, O>(input: I, output: O) -> Self
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
        tokio::spawn(async move { join!(forward_input_messages, forward_output_messages) });

        let service = BaseLsService {
            pending,
            output_tx: output_message_tx,
        };
        let socket = BaseLsSocket {
            unmatched_responses_rx,
            pending_requests: input_request_rx,
            was_previous_response: false,
        };
        Self { service, socket }
    }

    fn split(self) -> (Self::LsService, Self::LsSocket) {
        (self.service, self.socket)
    }
}
