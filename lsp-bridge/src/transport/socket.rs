use std::{
    collections::VecDeque,
    error::Error,
    fmt::{self, Display, Formatter},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use dashmap::{DashMap, Entry};
use futures::{
    Sink, SinkExt, channel::{mpsc::Sender, oneshot}, future::BoxFuture,
};
use tower_lsp_server::jsonrpc::{Id, Request, Response};

use crate::transport::handle::Message;

#[derive(Debug, Default)]
pub struct Pending(DashMap<Id, VecDeque<oneshot::Sender<Response>>>);

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

impl Pending {
    pub fn insert(&self, id: Id) -> oneshot::Receiver<Response> {
        let (response_tx, response_rx) = oneshot::channel();
        match self.0.entry(id) {
            Entry::Occupied(mut o) => o.get_mut().push_back(response_tx),
            Entry::Vacant(v) => {
                v.insert(vec![response_tx].into());
            }
        };
        response_rx
    }
}

impl Sink<Response> for &Pending {
    type Error = SendError;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(self: std::pin::Pin<&mut Self>, item: Response) -> Result<(), Self::Error> {
        match self.0.entry(item.id().clone()) {
            Entry::Occupied(mut o) => {
                o.get_mut()
                    .pop_front()
                    .unwrap()
                    .send(item)
                    .map_err(|_| SendError::Disconnected)?;
                if o.get().is_empty() {
                    o.remove();
                }
                Ok(())
            }
            _ => Err(SendError::UnknownId(item.into_parts().0)),
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

#[derive(Clone)]
pub struct LanguageServerSocket {
    request_tx: Sender<Request>,
    pending: Arc<Pending>,
}

impl LanguageServerSocket {
    pub fn new(request_tx: Sender<Request>, pending: Arc<Pending>) -> Self {
        LanguageServerSocket {
            request_tx,
            pending,
        }
    }

    pub fn insert(&self, id: Id) -> oneshot::Receiver<Response> {
        self.pending.insert(id)
    }

    pub fn send_request(&self, request: Request) -> BoxFuture<'_, Result<Option<oneshot::Receiver<Response>>, SendError>> {
        let rx = if let Some(id) = request.id().cloned() {
            Some(self.insert(id))
        } else {
            None
        };

        Box::pin(async move {
            self.request_tx.clone().send(request).await?;
            Ok(rx)
        })
    }
}

impl Sink<Message> for LanguageServerSocket {
    type Error = SendError;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let poll1 = self.request_tx.poll_ready_unpin(cx)?;
        let poll2 = self.pending.as_ref().poll_ready_unpin(cx)?;
        if poll1.is_pending() || poll2.is_pending() {
            return Poll::Pending;
        }
        Poll::Ready(Ok(()))
    }

    fn start_send(mut self: Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
        match item {
            Message::Request(request) => {
                self.request_tx.start_send_unpin(request)?;
            },
            Message::Response(response) => self.pending.as_ref().start_send_unpin(response)?,
        };
        Ok(())
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let poll1 = self.request_tx.poll_flush_unpin(cx)?;
        let poll2 = self.pending.as_ref().poll_flush_unpin(cx)?;
        if poll1.is_pending() || poll2.is_pending() {
            return Poll::Pending;
        }
        Poll::Ready(Ok(()))
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let poll1 = self.request_tx.poll_close_unpin(cx)?;
        let poll2 = self.pending.as_ref().poll_close_unpin(cx)?;
        if poll1.is_pending() || poll2.is_pending() {
            return Poll::Pending;
        }
        Poll::Ready(Ok(()))
    }
}
