use crate::{
    bridge::core::create_client, layer::log::LogLayer, transport::core::{LsHandle, LsService, LsSocket},
};
use tower::{Layer, Service};
use tower_lsp_server::{
    Client, ClientSocket,
    jsonrpc::{Request, Response},
};

use super::Bridge;

pub struct SimpleBridge<S> {
    inner: Bridge<S>,
}

impl<S> SimpleBridge<S> {
    pub fn new<Fun>(fun: Fun) -> (Self, ClientSocket)
    where
        Fun: FnOnce(Client) -> Self,
    {
        let (client, socket) = create_client();
        let bridge = fun(client);
        (bridge, socket)
    }

    pub fn client(&self) -> Client {
        self.inner.client.clone()
    }

    pub fn server(&self) -> S
    where
        S: Clone,
    {
        self.inner.server("main").unwrap().clone()
    }
}

impl<S> SimpleBridge<S> {
    pub fn connect<So>(client: Client, handle: LsHandle<S, So>) -> Self
    where
        So: LsSocket,
    {
        let service = handle.connect(client.clone()).schedule();
        Self {
            inner: Bridge::build(client).add_server("main", service),
        }
    }

    pub fn connect_with_log<So>(client: Client, handle: LsHandle<S, So>, target_name: &'static str) -> Self
    where
        So: LsSocket,
    {
        let log_layer = LogLayer::new("client", target_name, true);
        let client_log = log_layer.layer(client.clone());
        let service = handle.connect(client_log).schedule();
        Self {
            inner: Bridge::build(client).add_server("main", service),
        }
    }
}

impl<S> Service<Request> for SimpleBridge<S>
where
    S: Clone + LsService,
{
    type Response = Option<Response>;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.server().poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        self.server().call(req)
    }
}
