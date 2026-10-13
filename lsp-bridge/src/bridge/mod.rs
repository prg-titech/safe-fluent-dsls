pub mod core;
pub mod java_json;
pub mod router;

use futures::{SinkExt, channel::mpsc, future::BoxFuture};
use lazy_static::lazy_static;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use tower::Service;
use tower_lsp_server::{
    Loopback,
    jsonrpc::{Request, Response},
};

use crate::{
    layer::server::LsServer,
    transport::{Pending, error::ExitedError},
};

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

pub fn create_bridge(
    server_names: &[&'static str],
) -> (Bridge<StandardService>, BridgeSocket<StandardSocket>) {
    let (client, client_socket) = create_standard_service();
    let mut servers = HashMap::new();
    let mut sockets = HashMap::new();
    for &name in server_names {
        let (service, socket) = create_standard_service();
        servers.insert(name, LsServer::new(service, name));
        sockets.insert(name, socket);
    }
    let bridge = Bridge {
        inner: Arc::new(BridgeInner {
            client: LsServer::new(client, "client"),
            servers: Arc::new(servers),
        }),
    };
    let bridge_socket = BridgeSocket::new(client_socket, sockets);
    (bridge, bridge_socket)
}

pub struct BridgeInner<S> {
    client: LsServer<S>,
    servers: Arc<HashMap<&'static str, LsServer<S>>>,
}

pub struct Bridge<S> {
    inner: Arc<BridgeInner<S>>,
}

impl<S> Bridge<S> {
    pub fn new(client: S, servers: HashMap<&'static str, S>) -> Self {
        Self {
            inner: Arc::new(BridgeInner {
                client: LsServer::new(client, "bridge"),
                servers: Arc::new(
                    servers
                        .into_iter()
                        .map(|(n, s)| (n, LsServer::new(s, "bridge")))
                        .collect(),
                ),
            }),
        }
    }

    pub fn client(&self) -> LsServer<S>
    where
        S: Clone,
    {
        self.inner.client.clone()
    }

    pub fn server(&self, name: &str) -> Option<LsServer<S>>
    where
        S: Clone,
    {
        self.inner.servers.get(name).cloned()
    }
}

impl<S> Clone for Bridge<S> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

pub struct BridgeSocket<L> {
    client_socket: Option<L>,
    server_sockets: HashMap<&'static str, Option<L>>,
}

impl<L> BridgeSocket<L> {
    pub fn new(client_socket: L, server_sockets: HashMap<&'static str, L>) -> Self {
        Self {
            client_socket: Some(client_socket),
            server_sockets: server_sockets
                .into_iter()
                .map(|(n, s)| (n, Some(s)))
                .collect(),
        }
    }

    pub fn client_socket(&mut self) -> Option<L> {
        self.client_socket.take()
    }

    pub fn server_socket(&mut self, name: &str) -> Option<L> {
        self.server_sockets.get_mut(name).and_then(|s| s.take())
    }
}

pub fn create_standard_service() -> (StandardService, StandardSocket) {
    let (tx, rx) = mpsc::channel(0);
    let pending = Pending::default();
    (
        StandardService {
            inner: Arc::new(StandardServiceInner {
                tx,
                pending: pending.clone(),
            }),
        },
        StandardSocket { rx, pending },
    )
}

#[derive(Clone)]
pub struct StandardServiceInner {
    tx: mpsc::Sender<Request>,
    pending: Pending,
}

pub struct StandardService {
    inner: Arc<StandardServiceInner>,
}

impl Clone for StandardService {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl Service<Request> for StandardService {
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
        let response_rx = req
            .id()
            .cloned()
            .and_then(|id| self.inner.pending.insert(id));

        Box::pin(async move {
            if tx.send(req).await.is_err() {
                return Err(ExitedError);
            }

            if let Some(response_rx) = response_rx {
                match response_rx.await {
                    Ok(fut) => Ok(Some(fut)),
                    Err(_) => Err(ExitedError),
                }
            } else {
                Ok(None)
            }
        })
    }
}

pub struct StandardSocket {
    rx: mpsc::Receiver<Request>,
    pending: Pending,
}

impl Loopback for StandardSocket {
    type RequestStream = mpsc::Receiver<Request>;

    type ResponseSink = Pending;

    fn split(self) -> (Self::RequestStream, Self::ResponseSink) {
        let Self { rx, pending } = self;
        (rx, pending)
    }
}
