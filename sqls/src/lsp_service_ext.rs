use std::pin::Pin;

use futures::future::BoxFuture;
use futures::{FutureExt, SinkExt, StreamExt};
use futures_util::sink::With;
use futures_util::stream::Map;
use log::{error, info};
use tower::Service;
use tower_lsp_server::jsonrpc::{Request, Response};
use tower_lsp_server::{Client, ClientSocket, ExitedError, LanguageServer, Loopback, LspService};

pub struct LspServiceExt<S> {
    inner: LspService<S>,
}

impl<S: LanguageServer> LspServiceExt<S> {
    pub fn new<F>(init: F) -> (Self, ClientSocket)
    where
        F: FnOnce(Client) -> S,
    {
        let (inner, socket) = LspService::new(init);
        (Self { inner }, socket)
    }
}

impl<S: LanguageServer> Service<Request> for LspServiceExt<S> {
    type Response = Option<Response>;
    type Error = ExitedError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        info!("--> Request:  {}", serde_json::to_string(&req).unwrap());
        self.inner
            .call(req)
            .then(async move |result| match result {
                Ok(Some(response)) => {
                    info!(
                        "<-- Response: {}",
                        serde_json::to_string(&response).unwrap()
                    );
                    Ok(Some(response))
                }
                Ok(None) => {
                    Ok(None)
                }
                Err(error) => {
                    error!("Exited Error");
                    Err(error)
                }
            })
            .boxed()
    }
}

pub struct ClientSocketExt {
    inner: ClientSocket,
}

impl ClientSocketExt {
    pub fn new(inner: ClientSocket) -> Self {
        Self { inner }
    }
}

fn report_server_to_client_request(request: Request) -> Request {
    info!("<-- Request:  {}", serde_json::to_string(&request).unwrap());
    request
}

fn report_client_to_server_response(
    response: Response,
) -> Pin<Box<dyn Future<Output = Result<Response, ExitedError>>>> {
    info!(
        "--> Response: {}",
        serde_json::to_string(&response).unwrap()
    );

    let result = async |response| Ok(response);
    Box::pin(result(response))
}

impl Loopback for ClientSocketExt {
    type RequestStream = Map<<ClientSocket as Loopback>::RequestStream, fn(Request) -> Request>;

    type ResponseSink = With<
        <ClientSocket as Loopback>::ResponseSink,
        Response,
        Response,
        Pin<Box<dyn Future<Output = Result<Response, ExitedError>>>>,
        fn(Response) -> Pin<Box<dyn Future<Output = Result<Response, ExitedError>>>>,
    >;

    fn split(self) -> (Self::RequestStream, Self::ResponseSink) {
        let (request_stream, response_sink) = self.inner.split();

        let request_stream =
            request_stream.map(report_server_to_client_request as fn(Request) -> Request);
        let response_sink = response_sink.with(
            report_client_to_server_response
                as fn(Response) -> Pin<Box<dyn Future<Output = Result<Response, ExitedError>>>>,
        );

        (request_stream, response_sink)
    }
}
