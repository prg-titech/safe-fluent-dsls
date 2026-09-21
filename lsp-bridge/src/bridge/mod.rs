mod router;

use std::collections::HashMap;
use std::error::Error;
use std::task::{Context, Poll};

use futures::future::BoxFuture;
use futures::{Sink, join};
use tokio::io::{AsyncRead, AsyncWrite};
use tower::Service;
use tower_lsp_server::jsonrpc::{Request, Response};
use tower_lsp_server::{Client, Loopback, Server};

use self::router::Router;
use crate::bridge::router::BoxLsService;
use crate::transport::ExitedError;

pub struct Bridge {
    client: Router<Client>,
    main_server: Router<crate::server::Server<BoxLsService>>,
    extra_servers: HashMap<&'static str, Router<BoxLsService>>,
}

impl Bridge {
    pub fn new(client: Client, main_server: crate::server::Server<BoxLsService>) -> Self {
        Self {
            client: Router::new(client),
            main_server: Router::new(main_server),
            extra_servers: HashMap::new(),
        }
    }

    pub async fn serve<I, O, L>(self, input: I, output: O, loopback: L)
    where
        I: AsyncRead + Unpin,
        O: AsyncWrite,
        L: Loopback,
        <L::ResponseSink as Sink<Response>>::Error: Error,
    {
        let server = Server::new(input, output, loopback);
        let serve = server.serve(self.main_server);

        join!(serve);
    }
}

impl Service<Request> for Bridge {
    type Response = Option<Response>;
    type Error = ExitedError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.main_server.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        self.main_server.call(req)
    }
}
