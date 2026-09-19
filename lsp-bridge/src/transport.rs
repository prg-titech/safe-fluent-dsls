pub mod handle;

use std::ffi::OsStr;
use std::marker::PhantomData;
use std::process::Stdio;
use std::sync::Arc;
use std::task::Poll;
use std::{
    fmt::{self, Display, Formatter},
    pin::Pin,
};

use dashmap::{DashMap, Entry};
use futures::channel::mpsc::{self, UnboundedReceiver};
use futures::channel::oneshot;
use futures::{FutureExt, Sink, SinkExt, StreamExt};
use log::warn;
use tower::Service;
use tower_lsp_server::jsonrpc::{self, Id, Request, Response};

use crate::transport::handle::{LanguageServerHandle, Message, MpscHandle};

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

pub struct LanguageServerService<H, Tx> {
    request_tx: Tx,
    pending_requests: Arc<DashMap<Id, Vec<oneshot::Sender<Response>>>>,
    phantom_handle: PhantomData<H>,
}

impl<H> LanguageServerService<H, <H as LanguageServerHandle>::Requests>
where
    H: LanguageServerHandle,
{
    pub fn start(handle: H) -> (Self, UnboundedReceiver<Request>) {
        let (tx, responses_rx) = handle.split();
        let mut responses_rx = Box::pin(responses_rx);
        let (mut server_requests_tx, server_requests_rx) = mpsc::unbounded();
        let pending_requests = Arc::new(DashMap::<Id, Vec<oneshot::Sender<Response>>>::new());
        let _pending_requests = pending_requests.clone();

        tokio::spawn(async move {
            while let Some(m) = responses_rx.next().await {
                match m {
                    Message::Request(request) => {
                        if let Err(_) = server_requests_tx.send(request).await {
                            break;
                        }
                    }
                    Message::Response(response) => {
                        match _pending_requests.entry(response.id().clone()) {
                            Entry::Occupied(mut o) => {
                                let _ = o.get_mut().remove(0).send(response);
                                if o.get().is_empty() {
                                    o.remove();
                                }
                            }
                            Entry::Vacant(_) => warn!(
                                "Cannot map response to request: {}",
                                serde_json::to_string(&response).unwrap()
                            ),
                        }
                    }
                }
            }
        });

        (
            LanguageServerService {
                request_tx: tx,
                pending_requests,
                phantom_handle: PhantomData::default(),
            },
            server_requests_rx,
        )
    }
}

impl LanguageServerService<MpscHandle, <MpscHandle as LanguageServerHandle>::Requests> {
    pub fn stdio<S: AsRef<OsStr>>(
        command: S,
        args: &[S],
    ) -> Result<(Self, UnboundedReceiver<Request>), std::io::Error> {
        let mut child = tokio::process::Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let handle = MpscHandle::from_stdio(4, stdout, stdin);
        Ok(LanguageServerService::start(handle))
    }
}

impl<H> Service<Request> for LanguageServerService<H, <H as LanguageServerHandle>::Requests>
where
    H: LanguageServerHandle,
    <H as LanguageServerHandle>::Requests: Clone,
{
    type Response = Option<Response>;
    type Error = ExitedError;
    type Future = LsRequest<<H as LanguageServerHandle>::Requests>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        match self.request_tx.poll_ready_unpin(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(_)) => Poll::Ready(Err(ExitedError)),
        }
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let rx = if let Some(id) = req.id().cloned() {
            let (tx, rx) = oneshot::channel();
            match self.pending_requests.entry(id) {
                Entry::Occupied(mut v) => v.get_mut().push(tx),
                Entry::Vacant(v) => {
                    v.insert(vec![tx]);
                }
            }
            Some(rx)
        } else {
            None
        };

        LsRequest {
            request: Some(req),
            tx: self.request_tx.clone(),
            rx: rx,
            send_started: false,
            send_done: false,
        }
    }
}

pub struct LsRequest<Tx> {
    request: Option<Request>,
    tx: Tx,
    rx: Option<oneshot::Receiver<Response>>,
    send_started: bool,
    send_done: bool,
}

impl<Tx> Future for LsRequest<Tx>
where
    Tx: Sink<Request> + Unpin,
{
    type Output = Result<Option<Response>, ExitedError>;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if self.send_done {
            match self.rx.as_mut().unwrap().poll_unpin(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Ok(r)) => Poll::Ready(Ok(Some(r))),
                Poll::Ready(Err(_)) => Poll::Ready(Err(ExitedError)),
            }
        } else if self.send_started {
            match self.tx.poll_flush_unpin(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Ok(())) => {
                    if self.rx.is_none() {
                        return Poll::Ready(Ok(None));
                    }
                    self.send_done = true;
                    self.poll(cx)
                }
                Poll::Ready(Err(_)) => Poll::Ready(Err(ExitedError)),
            }
        } else {
            match self.tx.poll_ready_unpin(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Ok(())) => {
                    let request = self.request.take().unwrap();
                    if let Err(_) = self.tx.start_send_unpin(request) {
                        return Poll::Ready(Err(ExitedError));
                    }
                    self.send_started = true;
                    self.poll(cx)
                }
                Poll::Ready(Err(_)) => Poll::Ready(Err(ExitedError)),
            }
        }
    }
}
