mod router;

use std::collections::HashSet;
use std::task::Poll;

use dashmap::{DashMap, DashSet};
use futures::future::BoxFuture;
use futures::{FutureExt, StreamExt, TryFutureExt, join};
use lazy_static::lazy_static;
use tokio::sync::OwnedRwLockWriteGuard;
use tower::Service;
use tower_lsp_server::jsonrpc::{Id, Request, Response, Result as JrpcResult};
use tower_lsp_server::{Client, ClientSocket, LanguageServer};
use tower_lsp_server::{LspService, ls_types::*};

use crate::server::LsServer;
use crate::transport::{BaseLsHandle, BaseLsService, ExitedError, LsHandle, Message};

lazy_static! {
    static ref METHODS: HashSet<&'static str> = {
        [
            "initialize",
            "initialized",
            "shutdown",
            "textDocument/didOpen",
            "textDocument/didChange",
            "textDocument/didClose",
        ]
        .into_iter()
        .collect()
    };
}

#[derive(Debug, Clone)]
pub struct BridgeInner {
    client: Client,
    main_server: LsServer<BaseLsService>,
    embedded_server: LsServer<BaseLsService>,

    // The following is just for testing
    json_files: DashSet<Uri>,
}

impl BridgeInner {
    pub fn new(client: Client, main_server: BaseLsHandle, embedded_server: BaseLsHandle) -> Self {
        let mut _client = client.clone();
        let (main_server, mut main_socket) = LsHandle::split(main_server);
        let (embedded_server, mut embedded_socket) = LsHandle::split(embedded_server);
        let forward_main_socket = async move {
            while let Some(m) = main_socket.next().await {
                match m {
                    Message::Request(request) => {
                        let _ = _client.call(request).await;
                    }
                    Message::Response(_) => (),
                }
            }
        };
        let mut _client = client.clone();
        let forward_embedded_socket = async move {
            while let Some(m) = embedded_socket.next().await {
                match m {
                    Message::Request(request) => {
                        let (method, id, params) = request.into_parts();
                        let id = id.map(|id| Id::String(format!("embedded:{}", id.to_string())));
                        let mut request_builder = Request::build(method);
                        if let Some(id) = id {
                            request_builder = request_builder.id(id);
                        }
                        if let Some(params) = params {
                            request_builder = request_builder.params(params);
                        }
                        let request = request_builder.finish();
                        let _ = _client.call(request).await;
                    }
                    Message::Response(_) => (),
                }
            }
        };
        tokio::spawn(async { join!(forward_main_socket, forward_embedded_socket) });

        let main_server = LsServer::new(main_server);
        let embedded_server = LsServer::new(embedded_server);

        Self {
            client,
            main_server,
            embedded_server,
            json_files: DashSet::new(),
        }
    }
}

impl LanguageServer for BridgeInner {
    async fn initialize(&self, params: InitializeParams) -> JrpcResult<InitializeResult> {
        let _ = self.embedded_server.initialize(params.clone()).await;
        self.main_server.initialize(params).await
    }

    async fn initialized(&self, params: InitializedParams) {
        self.embedded_server.initialized(params).await;
        self.main_server.initialized(params).await
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        if params.text_document.language_id == "json" || params.text_document.language_id == "jsonc"
        {
            self.json_files.insert(params.text_document.uri.clone());
            self.embedded_server.did_open(params).await
        } else {
            self.main_server.did_open(params).await
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if self.json_files.contains(&params.text_document.uri) {
            self.embedded_server.did_change(params).await;
        } else {
            self.main_server.did_change(params).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        if let Some(_) = self.json_files.remove(&params.text_document.uri) {
            self.embedded_server.did_close(params).await;
        } else {
            self.main_server.did_close(params).await;
        }
    }

    async fn shutdown(&self) -> JrpcResult<()> {
        let _ = self.embedded_server.shutdown().await;
        self.main_server.shutdown().await
    }
}

#[derive(Debug)]
pub struct BridgeBuilder {
    main_server: Option<BaseLsHandle>,
    embedded_server: Option<BaseLsHandle>,
}

impl BridgeBuilder {
    pub fn main_server(mut self, service: BaseLsHandle) -> Self {
        self.main_server = Some(service);
        self
    }

    pub fn embedded_server(mut self, service: BaseLsHandle) -> Self {
        self.embedded_server = Some(service);
        self
    }

    pub fn build(self) -> Option<(Bridge, ClientSocket)> {
        let Self {
            main_server,
            embedded_server,
        } = self;
        match (main_server, embedded_server) {
            (Some(main_server), Some(embedded_server)) => {
                let (lsp_service, client_socket) = LspService::new(|client| {
                    BridgeInner::new(client, main_server, embedded_server)
                });
                Some((Bridge { inner: lsp_service }, client_socket))
            }
            _ => None,
        }
    }
}

impl Default for BridgeBuilder {
    fn default() -> Self {
        Self {
            main_server: Default::default(),
            embedded_server: Default::default(),
        }
    }
}

impl Bridge {
    pub fn builder() -> BridgeBuilder {
        BridgeBuilder::default()
    }
}

pub struct Bridge {
    inner: LspService<BridgeInner>,
}

impl Service<Request> for Bridge {
    type Response = Option<Response>;

    type Error = ExitedError;

    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::prelude::v1::Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(|e| e.into())
    }

    fn call(&mut self, req: Request) -> Self::Future {
        if METHODS.contains(req.method()) {
            Box::pin(self.inner.call(req).map_err(|e| e.into()))
        } else {
            Box::pin(BridgeFuture {
                request: Some(req),
                server: self.inner.inner().main_server.clone(),
                guard: None,
                is_ready: false,
                future: None,
            })
        }
    }
}

pub struct BridgeFuture {
    request: Option<Request>,
    server: LsServer<BaseLsService>,
    guard: Option<OwnedRwLockWriteGuard<BaseLsService>>,
    is_ready: bool,
    future: Option<<BaseLsService as Service<Request>>::Future>,
}

impl Future for BridgeFuture {
    type Output = Result<Option<Response>, ExitedError>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if let Some(future) = &mut self.future {
            future.poll_unpin(cx).map_err(|e| e.into())
        } else if let Some(mut guard) = self.guard.take() {
            if self.is_ready {
                self.future = Some(guard.call(self.request.take().unwrap()));
                self.poll_unpin(cx)
            } else {
                match guard.poll_ready(cx) {
                    Poll::Pending => {
                        self.guard = Some(guard);
                        Poll::Pending
                    }
                    Poll::Ready(Ok(())) => {
                        self.guard = Some(guard);
                        self.poll_unpin(cx)
                    }
                    Poll::Ready(Err(e)) => Poll::Ready(Err(e.into())),
                }
            }
        } else {
            match self.server.poll_write(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(guard) => {
                    self.guard = Some(guard);
                    self.poll_unpin(cx)
                }
            }
        }
    }
}
