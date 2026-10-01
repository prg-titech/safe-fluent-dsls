pub mod core;
pub mod error;

use std::{
    collections::VecDeque,
    ffi::OsStr,
    pin::Pin,
    process::Stdio,
    sync::Arc,
    task::{Context, Poll},
};

use dashmap::{DashMap, Entry};
use futures::{
    FutureExt, Sink, SinkExt, StreamExt,
    channel::{mpsc, oneshot},
    future::{self, BoxFuture},
};
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    process::{ChildStdin, ChildStdout, Command},
};
use tokio_util::codec::{FramedRead, FramedWrite};
use tower::Service;
use tower_lsp_server::jsonrpc::{Id, Request, Response};

use crate::{
    codec::LanguageServerCodec,
    transport::{
        core::{LsHandle, LsSocket, Schedulable, Task},
        error::{ExitedError, SendError},
    },
};

#[derive(Deserialize, Serialize)]
#[cfg_attr(test, derive(Debug, PartialEq))]
#[serde(untagged)]
pub enum Message {
    /// A response message.
    Response(Response),
    /// A request or notification message.
    Request(Request),
}

pub struct Server<I, O> {
    input: I,
    output: O,
}

impl<I, O> Server<I, O> {
    pub fn new(input: I, output: O) -> Self {
        Self { input, output }
    }

    pub fn frame(
        self,
    ) -> (
        FramedRead<I, LanguageServerCodec<Message>>,
        FramedWrite<O, LanguageServerCodec<Message>>,
    ) {
        let framed_read = FramedRead::new(self.input, LanguageServerCodec::default());
        let framed_write = FramedWrite::new(self.output, LanguageServerCodec::default());
        (framed_read, framed_write)
    }
}

impl Server<ChildStdout, ChildStdin> {
    pub fn spawn<S: AsRef<OsStr>, I: IntoIterator<Item = O>, O: AsRef<OsStr>>(
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
        Ok(Self { input, output })
    }
}

#[derive(Clone)]
pub struct BaseLsService {
    pending: Pending,
    request_tx: mpsc::Sender<Request>,
}

impl Service<Request> for BaseLsService {
    type Response = Option<Response>;
    type Error = ExitedError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.request_tx
            .poll_ready_unpin(cx)
            .map_err(|e| <mpsc::SendError as Into<ExitedError>>::into(e))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let mut this = self.clone();
        async move {
            let response_rx = req.id().cloned().and_then(|id| this.pending.insert(id));
            this.request_tx.send(req).await?;
            match response_rx {
                Some(response_rx) => Ok(response_rx.await.map(|r| Some(r))?),
                None => Ok(None),
            }
        }
        .boxed()
    }
}

pub struct BaseLsSocket {
    request_rx: mpsc::Receiver<Request>,
    response_tx: mpsc::Sender<Response>,
}

impl LsSocket for BaseLsSocket {
    type RequestStream = mpsc::Receiver<Request>;
    type ResponseSink = mpsc::Sender<Response>;

    fn split(self) -> (Self::RequestStream, Self::ResponseSink) {
        let Self {
            request_rx,
            response_tx,
        } = self;
        (request_rx, response_tx)
    }
}

impl LsHandle<BaseLsService, BaseLsSocket> {
    pub fn from_server<I, O>(server: Server<I, O>) -> Schedulable<Self, Task<()>>
    where
        I: AsyncRead + Unpin + Send + 'static,
        O: AsyncWrite + Unpin + Send + 'static,
    {
        let (mut framed_read, framed_write) = server.frame();
        let pending = Pending::default();
        let mut input_response_tx = pending.clone();
        let (mut input_request_tx, input_request_rx) = mpsc::channel(0);
        let forward_input_messages = async move {
            while let Some(message) = framed_read.next().await {
                match message {
                    Ok(Message::Request(request)) => {
                        let _ = input_request_tx.send(request).await?;
                    }
                    Ok(Message::Response(response)) => {
                        input_response_tx.send(response).await?;
                    }
                    Err(_) => (),
                }
            }
            Ok::<_, SendError>(())
        };

        let (output_request_tx, output_request_rx) = mpsc::channel(0);
        let (output_response_tx, output_response_rx) = mpsc::channel(0);
        let forward_output_messages = output_request_rx
            .map(|request| Message::Request(request))
            .chain(output_response_rx.map(|response| Message::Response(response)))
            .map(|m| Ok(m))
            .forward(framed_write);

        let task = future::join(forward_input_messages, forward_output_messages).boxed();

        let service = BaseLsService {
            pending,
            request_tx: output_request_tx,
        };
        let socket = BaseLsSocket {
            request_rx: input_request_rx,
            response_tx: output_response_tx,
        };
        let handle = Self::new(service, socket);
        Schedulable::new(handle, task).map_task(|_| ())
    }
}

#[derive(Debug, Clone, Default)]
pub struct Pending {
    pending: Arc<DashMap<Id, VecDeque<oneshot::Sender<Response>>>>,
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
            Err(SendError::UnknownId(item.id().clone()))
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
