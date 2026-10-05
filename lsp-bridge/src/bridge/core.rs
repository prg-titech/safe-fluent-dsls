// TODO: remove this line
#![allow(dead_code)]

pub mod simple;

use std::sync::Arc;

use dashmap::{DashMap, mapref::one::Ref};
use futures::{FutureExt, SinkExt, channel::mpsc::Sender, future::BoxFuture};
use log::error;
use tower::Service;
use tower_lsp_server::{
    Client, ClientSocket, LanguageServer, LspService,
    jsonrpc::{self, Request, Response},
    ls_types::{InitializeParams, InitializeResult, notification, request},
};

use crate::{
    layer::UniqueIdIter,
    transport::{Pending, error::ExitedError},
};

struct DummyLanguageServer {}

impl LanguageServer for DummyLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> jsonrpc::Result<InitializeResult> {
        todo!()
    }

    async fn shutdown(&self) -> jsonrpc::Result<()> {
        todo!()
    }
}

pub fn create_client() -> (Client, ClientSocket) {
    let mut client = None;
    let client_ref = &mut client;
    let (_, socket) = LspService::new(move |client| {
        *client_ref = Some(client);
        DummyLanguageServer {}
    });
    (client.unwrap(), socket)
}

pub struct ServerInner {
    tx: Sender<Request>,
    request_ids: UniqueIdIter,
    pending: Pending,
}

#[derive(Clone)]
pub struct Server {
    inner: Arc<ServerInner>,
}

impl Server {
    pub fn new(tx: Sender<Request>, pending: Pending) -> Server {
        Server {
            inner: Arc::new(ServerInner {
                tx,
                request_ids: UniqueIdIter::new(),
                pending,
            }),
        }
    }

    pub async fn send_notification<N>(&self, params: N::Params)
    where
        N: notification::Notification,
    {
        let request = Request::build(N::METHOD)
            .params(serde_json::to_value(params).unwrap())
            .finish();
        if self.clone().call(request).await.is_err() {
            error!("failed to send notification");
        }
    }

    pub async fn send_request<R>(&self, params: R::Params) -> jsonrpc::Result<R::Result>
    where
        R: request::Request,
    {
        let id = self.inner.request_ids.clone().next().unwrap();
        let request = Request::build(R::METHOD)
            .id(id)
            .params(serde_json::to_value(params).unwrap())
            .finish();

        let Ok(Some(response)) = self.clone().call(request).await else {
            return Err(jsonrpc::Error::internal_error());
        };

        let (_, result) = response.into_parts();
        result.and_then(|v| {
            serde_json::from_value(v).map_err(|e| jsonrpc::Error {
                code: jsonrpc::ErrorCode::ParseError,
                message: e.to_string().into(),
                data: None,
            })
        })
    }
}

impl Service<Request> for Server {
    type Response = Option<Response>;
    type Error = ExitedError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.tx.clone().poll_ready(cx).map_err(|e| e.into())
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let mut tx = self.inner.tx.clone();
        let rx = req
            .id()
            .cloned()
            .and_then(|id| self.inner.pending.insert(id));

        async move {
            if tx.send(req).await.is_err() {
                return Err(ExitedError);
            }
            if let Some(rx) = rx {
                Ok(Some(rx.await?))
            } else {
                Ok(None)
            }
        }
        .boxed()
    }
}

#[derive(Clone)]
pub struct Bridge<S> {
    client: Client,
    servers: Arc<DashMap<&'static str, S>>,
}

impl<S> Bridge<S> {
    pub fn build(client: Client) -> Self {
        Self {
            client,
            servers: Arc::default(),
        }
    }

    pub fn add_server(self, name: &'static str, server: S) -> Self {
        assert_ne!("client", name);
        self.servers.insert(name, server);
        self
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn server(&self, name: &str) -> Option<Ref<'_, &'static str, S>> {
        self.servers.get(name)
    }
}
