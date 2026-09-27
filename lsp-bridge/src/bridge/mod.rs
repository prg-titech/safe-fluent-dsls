use std::collections::HashSet;
use std::fmt::Display;
use std::task::Poll;

use dashmap::DashSet;
use futures::future::BoxFuture;
use futures::{FutureExt, StreamExt, TryFutureExt, join};
use lazy_static::lazy_static;
use serde::Serialize;
use tokio::sync::OwnedRwLockWriteGuard;
use tower::util::BoxService;
use tower::{Layer, Service, ServiceExt};
use tower_lsp_server::jsonrpc::{self, Id, Request, Response, Result as JrpcResult};
use tower_lsp_server::{Client, ClientSocket, LanguageServer};
use tower_lsp_server::{LspService, ls_types::*};

use crate::logging::{LogLayer};
use crate::server::LsServer;
use crate::transport::{ExitedError, LsHandle, Message};

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

#[derive(Debug)]
pub struct BridgeInner<S> {
    #[allow(unused)]
    client: Client,
    main_server: LsServer<S>,
    embedded_server: LsServer<S>,

    // The following is just for testing
    json_files: DashSet<Uri>,
}

impl<S> Clone for BridgeInner<S> {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            main_server: self.main_server.clone(),
            embedded_server: self.embedded_server.clone(),
            json_files: self.json_files.clone(),
        }
    }
}

impl<S> BridgeInner<S> {
    pub fn new(client: Client, main_server: S, embedded_server: S) -> Self
where {
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

impl<S> LanguageServer for BridgeInner<S>
where
    S: Service<Request, Response = Option<Response>> + Sync + Send + 'static,
    S::Error: Into<jsonrpc::Error>,
    S::Future: Send,
{
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
pub struct BridgeBuilder<Hdl> {
    main_server: Option<Hdl>,
    embedded_server: Option<Hdl>,
    client_log: Option<LogLayer>,
    main_server_log: Option<LogLayer>,
    embedded_server_log: Option<LogLayer>,
}

fn try_add_log_layer<Rq: Serialize, S>(
    service: S,
    log_layer: Option<LogLayer>,
) -> BoxService<Rq, S::Response, S::Error>
where
    Rq: Serialize + 'static,
    S: Service<Rq> + Send + Sync + 'static,
    S::Response: Serialize + Send + 'static,
    S::Error: Display + Send + 'static,
    S::Future: Send + 'static,
{
    match log_layer {
        Some(log_layer) => log_layer.layer(service).boxed(),
        None => service.boxed(),
    }
    .boxed()
}

impl<Hdl> BridgeBuilder<Hdl> {
    pub fn log_client(mut self, log_layer: LogLayer) -> Self {
        self.client_log = Some(log_layer);
        self
    }

    pub fn main_server(mut self, service: Hdl) -> Self {
        self.main_server = Some(service);
        self
    }

    pub fn log_main_server(mut self, log_layer: LogLayer) -> Self {
        self.main_server_log = Some(log_layer);
        self
    }

    pub fn embedded_server(mut self, service: Hdl) -> Self {
        self.embedded_server = Some(service);
        self
    }

    pub fn log_embedded_server(mut self, log_layer: LogLayer) -> Self {
        self.embedded_server_log = Some(log_layer);
        self
    }

    pub fn build(
        self,
    ) -> (
        Bridge<BoxService<Request, Option<Response>, ExitedError>>,
        ClientSocket,
    )
    where
        Hdl: LsHandle + Send + Sync + 'static,
        Hdl::LsService: Send + Sync + Unpin,
        Hdl::LsSocket: Unpin + Send + Sync,
        <Hdl::LsService as Service<Request>>::Future: Send,
    {
        let Self {
            main_server,
            embedded_server,
            client_log,
            main_server_log,
            embedded_server_log,
        } = self;
        let (main_server, mut main_socket) = LsHandle::split(main_server.unwrap());
        let main_server = try_add_log_layer(main_server, main_server_log);

        let (embedded_server, mut embedded_socket) = LsHandle::split(embedded_server.unwrap());
        let embedded_server = try_add_log_layer(embedded_server, embedded_server_log);

        let (lsp_service, client_socket) = LspService::new(move |client| {
            let mut _client = client.clone();
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
                            let id =
                                id.map(|id| Id::String(format!("embedded:{}", id.to_string())));
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

            BridgeInner::new(client, main_server, embedded_server)
        });

        (Bridge { inner: lsp_service }, client_socket)
    }
}

impl<Hdl> Default for BridgeBuilder<Hdl> {
    fn default() -> Self {
        Self {
            main_server: Default::default(),
            embedded_server: Default::default(),
            client_log: Default::default(),
            main_server_log: Default::default(),
            embedded_server_log: Default::default()
        }
    }
}

pub struct Bridge<S> {
    inner: LspService<BridgeInner<S>>,
}

impl<S> Service<Request> for Bridge<S>
where
    S: Service<Request, Response = Option<Response>> + Sync + Send + Unpin + 'static,
    S::Error: Into<jsonrpc::Error> + Into<ExitedError>,
    S::Future: Send + Unpin,
{
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

pub struct BridgeFuture<S: Service<Request>> {
    request: Option<Request>,
    server: LsServer<S>,
    guard: Option<OwnedRwLockWriteGuard<S>>,
    is_ready: bool,
    future: Option<<S as Service<Request>>::Future>,
}

impl<S: Service<Request, Response = Option<Response>, Error = E>, E: Into<ExitedError>> Future
    for BridgeFuture<S>
where
    S: Unpin,
    S::Future: Unpin,
{
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
