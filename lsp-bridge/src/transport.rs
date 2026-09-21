pub mod handle;
pub mod socket;

use crate::transport::handle::{LanguageServerHandle, Message, MpscHandle};
use crate::transport::socket::{LanguageServerSocket, Pending, SendError};
use futures::channel::mpsc::{self, UnboundedReceiver};
use futures::channel::oneshot;
use futures::{FutureExt, SinkExt, Stream, StreamExt, join};
use std::ffi::OsStr;
use std::mem;
use std::process::Stdio;
use std::sync::Arc;
use std::task::Poll;
use std::{
    fmt::{self, Display, Formatter},
    pin::Pin,
};
use tower::Service;
use tower_lsp_server::jsonrpc::{self, Request, Response};

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

pub struct LanguageServerService {
    request_rx: UnboundedReceiver<Request>,
    socket: LanguageServerSocket,
}

impl LanguageServerService {
    pub fn stdio<S: AsRef<OsStr>>(command: S, args: &[S]) -> Result<Self, std::io::Error> {
        let mut child = tokio::process::Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let handle = MpscHandle::from_stdio(4, stdout, stdin);
        let (handle_request_tx, mut message_rx) = LanguageServerHandle::split(handle);
        let (socket_request_tx, socket_request_rx) = mpsc::channel(0);
        let (mut request_tx, request_rx) = mpsc::unbounded::<Request>();
        let (mut response_tx, response_rx) = mpsc::channel(0);

        let socket = LanguageServerSocket::new(socket_request_tx, Arc::new(Pending::default()));
        let _socket = socket.clone();

        tokio::spawn(async move {
            let forward_requests = socket_request_rx.map(|r| Ok(r)).forward(handle_request_tx);
            let forward_responses = response_rx
                .map(|r| Ok(Message::Response(r)))
                .forward(_socket);
            let forward_messages = async {
                while let Ok(message) = message_rx.recv().await {
                    match message {
                        Message::Request(request) => {
                            if request_tx.send(request).await.is_err() {
                                break;
                            }
                        }
                        Message::Response(response) => {
                            if response_tx.send(response).await.is_err() {
                                break;
                            }
                        }
                    }
                }
            };

            join!(forward_requests, forward_responses, forward_messages)
        });
        Ok(Self { request_rx, socket })
    }
}

impl Service<Request> for LanguageServerService {
    type Response = Option<Response>;
    type Error = ExitedError;
    type Future = LsRequest;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        match self.socket.poll_ready_unpin(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(_)) => Poll::Ready(Err(ExitedError)),
        }
    }

    fn call(&mut self, req: Request) -> Self::Future {
        LsRequest {
            socket: self.socket.clone(),
            state: LsRequestState::SendPending(req),
        }
    }
}

impl Stream for LanguageServerService {
    type Item = Request;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        self.request_rx.poll_next_unpin(cx)
    }
}

enum LsRequestState {
    SendPending(Request),
    SendStarted(Option<oneshot::Receiver<Response>>),
    ReceivePending(oneshot::Receiver<Response>),
    TemporaryEmpty,
}

impl LsRequestState {
    pub fn take(&mut self) -> LsRequestState {
        mem::replace(self, LsRequestState::TemporaryEmpty)
    }
}

pub struct LsRequest {
    socket: LanguageServerSocket,
    state: LsRequestState,
}

impl Future for LsRequest {
    type Output = Result<Option<Response>, ExitedError>;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let (next_state, poll) = match self.state.take() {
            LsRequestState::SendPending(request) => {
                let rx = request.id().cloned().map(|id| self.socket.insert(id));
                self.socket
                    .start_send_unpin(Message::Request(request.to_owned()))?;
                (LsRequestState::SendStarted(rx), None)
            }
            LsRequestState::SendStarted(rx) => match (self.socket.poll_flush_unpin(cx), rx) {
                (Poll::Pending, rx) => (LsRequestState::SendStarted(rx), Some(Poll::Pending)),
                (Poll::Ready(Ok(_)), Some(rx)) => (LsRequestState::ReceivePending(rx), None),
                (Poll::Ready(Ok(_)), None) => return Poll::Ready(Ok(None)),
                (Poll::Ready(Err(_)), _) => return Poll::Ready(Err(ExitedError)),
            },
            LsRequestState::ReceivePending(mut rx) => match rx.poll_unpin(cx) {
                Poll::Pending => (LsRequestState::ReceivePending(rx), Some(Poll::Pending)),
                Poll::Ready(result) => {
                    return Poll::Ready(result.map(|r| Some(r)).map_err(|_| ExitedError));
                }
            },
            LsRequestState::TemporaryEmpty => unreachable!(),
        };
        self.state = next_state;
        poll.unwrap_or_else(|| self.poll(cx))
    }
}
