pub mod core;
pub mod simple;

use lazy_static::lazy_static;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::layer::server::LsServer;

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
        S: Clone
    {   
        self.inner.client.clone()
    }

    pub fn server(&self, name: &str) -> Option<LsServer<S>>
    where 
        S: Clone
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
    client_socket: L,
    server_sockets: HashMap<&'static str, L>
}

impl<L> BridgeSocket<L> {
    pub fn new(client_socket: L, server_sockets: HashMap<&'static str, L>) -> Self {
        Self { client_socket, server_sockets }
    }
}

