pub mod core;

use crate::id_layer::map_id;
use crate::logging::LogLayer;
use crate::server::LsServer;
use crate::transport::core::{BoxLsService, LsHandle, LsService, Schedulable, Task};
use crate::transport::error::ExitedError;
use crate::transport::{BaseLsSocket, Server};
use dashmap::DashSet;
use futures::future::BoxFuture;
use futures::{FutureExt, TryFutureExt};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::ffi::{OsStr, OsString};
use std::io;
use std::task::Poll;
use tokio::sync::OwnedRwLockWriteGuard;
use tower::{Layer, Service, ServiceExt};
use tower_lsp_server::jsonrpc::{self, Request, Response, Result as JrpcResult};
use tower_lsp_server::ls_types::notification::Notification;
use tower_lsp_server::{Client, ClientSocket, LanguageServer};
use tower_lsp_server::{LspService, ls_types::*};

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
pub struct ServerBuilder {
    name: &'static str,
    command: OsString,
    args: Vec<OsString>,
    server_log: bool,
    client_log: bool,
}

impl ServerBuilder {
    pub fn new<S1: AsRef<OsStr>, S2: AsRef<OsStr>, I: IntoIterator<Item = S2>>(
        name: &'static str,
        command: S1,
        args: I,
    ) -> ServerBuilder {
        Self {
            name,
            command: command.as_ref().to_os_string(),
            args: args
                .into_iter()
                .map(|o| o.as_ref().to_os_string())
                .collect(),
            server_log: false,
            client_log: false,
        }
    }

    pub fn log_server(mut self) -> Self {
        self.server_log = true;
        self
    }

    pub fn log_client(mut self) -> Self {
        self.client_log = true;
        self
    }

    pub fn build(self) -> Result<Schedulable<NamedServer, Task<()>>, io::Error> {
        let server = Server::spawn(self.command, self.args)?;
        let handle = LsHandle::from_server(server)
            .map(|h| {
                if self.server_log {
                    h.layer(LogLayer::new("bridge", self.name, false)).boxed()
                } else {
                    h.boxed()
                }
            })
            .map(|handle| NamedServer::new(self.name, handle, self.client_log));
        Ok(handle)
    }
}

pub struct NamedServer {
    name: &'static str,
    handle: LsHandle<BoxLsService<ExitedError>, BaseLsSocket>,
    client_log: bool,
}

impl NamedServer {
    pub fn new(
        name: &'static str,
        handle: LsHandle<BoxLsService<ExitedError>, BaseLsSocket>,
        client_log: bool,
    ) -> NamedServer {
        Self {
            name,
            handle,
            client_log,
        }
    }

    pub fn connect_to_client(
        self,
        client: Client,
    ) -> Schedulable<BoxLsService<ExitedError>, Task<()>> {
        let Self {
            name,
            handle,
            client_log,
        } = self;
        let client = map_id(name).layer(client);
        let client = if client_log {
            LogLayer::new("client", name, true).layer(client).boxed()
        } else {
            client.boxed()
        };
        handle.connect(client)
    }
}

#[derive(Debug, Default)]
pub struct BridgeBuilder {
    main_server: Option<ServerBuilder>,
    embedded_server: Option<ServerBuilder>,
}

impl BridgeBuilder {
    pub fn main_server<S1: AsRef<OsStr>, S2: AsRef<OsStr>, I: IntoIterator<Item = S2>>(
        mut self,
        name: &'static str,
        command: S1,
        args: I,
    ) -> Self {
        self.main_server = Some(ServerBuilder::new(name, command, args));
        self
    }

    pub fn log_main_server(mut self) -> Self {
        self.main_server = Some(self.main_server.unwrap().log_server());
        self
    }

    pub fn log_main_client(mut self) -> Self {
        self.main_server = Some(self.main_server.unwrap().log_client());
        self
    }

    pub fn embedded_server<S1: AsRef<OsStr>, S2: AsRef<OsStr>, I: IntoIterator<Item = S2>>(
        mut self,
        name: &'static str,
        command: S1,
        args: I,
    ) -> Self {
        self.embedded_server = Some(ServerBuilder::new(name, command, args));
        self
    }

    pub fn log_embedded_server(mut self) -> Self {
        self.embedded_server = Some(self.embedded_server.unwrap().log_server());
        self
    }

    pub fn log_embedded_client(mut self) -> Self {
        self.embedded_server = Some(self.embedded_server.unwrap().log_client());
        self
    }

    pub fn build(self) -> Schedulable<(Bridge<BoxLsService<ExitedError>>, ClientSocket), Task<()>> {
        let Self {
            main_server,
            embedded_server,
        } = self;
        let main_server = main_server.unwrap();
        let embedded_server = embedded_server.unwrap();

        let mut lsp_task = None;
        let lsp_task_ref = &mut lsp_task;
        let (lsp_service, client_socket) = LspService::new(move |client| {
            let main_server = main_server
                .build()
                .unwrap()
                .then(|s| s.connect_to_client(client.clone()));
            let embedded_server = embedded_server
                .build()
                .unwrap()
                .then(|s| s.connect_to_client(client.clone()));

            let ((main_server, embedded_server), task) =
                main_server.join(embedded_server).into_parts();
            *lsp_task_ref = Some(task);
            BridgeInner {
                client,
                main_server: LsServer::new(main_server),
                embedded_server: LsServer::new(embedded_server),
                json_files: DashSet::new(),
            }
        });

        let result = (Bridge { inner: lsp_service }, client_socket);
        Schedulable::new(result, lsp_task.unwrap().map(|_| ()).boxed())
    }
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

pub enum Status {}

impl Notification for Status {
    type Params = StatusParams;

    const METHOD: &'static str = "language/status";
}
#[derive(Default, Serialize, Deserialize)]
pub struct StatusParams {
    message: String,
    r#type: String,
}

impl<S> LanguageServer for BridgeInner<S>
where
    S: LsService,
    S::Error: Into<jsonrpc::Error>,
    S::Future: Send,
{
    async fn initialize(&self, params: InitializeParams) -> JrpcResult<InitializeResult> {
        let _ = self.embedded_server.initialize(params.clone()).await;
        self.client
            .send_notification::<Status>(StatusParams {
                message: "Test...".to_owned(),
                ..Default::default()
            })
            .await;
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
