use crate::transport::ExitedError;
use futures::{Stream, StreamExt, future::BoxFuture};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    borrow::Cow, pin::Pin, sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    }, task::Poll,
};
use tokio::sync::RwLock;
use tower::Service;
use tower_lsp_server::{
    LanguageServer,
    jsonrpc::{self, Request, Response, Result as JrpcResult},
    ls_types::*,
};

#[derive(Debug)]
pub struct Server<S> {
    service: Arc<RwLock<S>>,
    current_id: Arc<AtomicU32>,
}

impl<S> Clone for Server<S> {
    fn clone(&self) -> Self {
        Self { service: self.service.clone(), current_id: self.current_id.clone() }
    }
}

impl<S> Server<S>
where
    S: Service<Request, Response = Option<Response>, Error = ExitedError> + Send + Sync + 'static,
{
    pub fn new(socket: S) -> Self {
        Self {
            service: Arc::new(RwLock::new(socket)),
            current_id: Arc::new(AtomicU32::new(0)),
        }
    }

    pub async fn exit(&self) -> JrpcResult<()> {
        self.service
            .write()
            .await
            .call(Self::create_simple_notification("exit"))
            .await?;
        Ok(())
    }

    fn create_simple_request<M>(&self, method: M) -> Request
    where
        M: Into<Cow<'static, str>>,
    {
        Request::build(method)
            .id(self.current_id.fetch_add(1, Ordering::SeqCst) as i64)
            .finish()
    }

    fn create_request<P, M>(&self, method: M, params: P) -> Request
    where
        P: Serialize,
        M: Into<Cow<'static, str>>,
    {
        Request::build(method)
            .params(serde_json::to_value(params).unwrap())
            .id(self.current_id.fetch_add(1, Ordering::SeqCst) as i64)
            .finish()
    }

    fn create_notification<P, M>(method: M, params: P) -> Request
    where
        P: Serialize,
        M: Into<Cow<'static, str>>,
    {
        Request::build(method)
            .params(serde_json::to_value(params).unwrap())
            .finish()
    }

    fn create_simple_notification<M>(method: M) -> Request
    where
        M: Into<Cow<'static, str>>,
    {
        Request::build(method).finish()
    }

    fn process_response<V: for<'a> Deserialize<'a>>(response: Option<Response>) -> JrpcResult<V> {
        let response = response.map(|r| r.into_parts().1);
        match response {
            None => Err(jsonrpc::Error {
                code: jsonrpc::ErrorCode::ParseError,
                message: "Expected a result, got nothing".into(),
                data: None,
            }),
            Some(Err(e)) => Err(e),
            Some(Ok(v)) => match serde_json::from_value(v) {
                Ok(v) => Ok(v),
                Err(e) => Err(jsonrpc::Error {
                    code: jsonrpc::ErrorCode::ParseError,
                    message: "Unable to parse the given value".into(),
                    data: Some(json!({
                        "line": e.line(),
                        "column": e.column(),
                        "message": e.to_string()
                    })),
                }),
            },
        }
    }
}

impl<S> LanguageServer for Server<S>
where
    S: Service<Request, Response = Option<Response>, Error = ExitedError> + Send + Sync + 'static,
    <S as Service<Request>>::Future: Send,
{
    async fn initialize(&self, params: InitializeParams) -> JrpcResult<InitializeResult> {
        let response = self
            .service
            .write()
            .await
            .call(self.create_request("initialize", params))
            .await?;
        Self::process_response(response)
    }

    async fn initialized(&self, params: InitializedParams) {
        let _ = self
            .service
            .write()
            .await
            .call(Self::create_notification("initialized", params))
            .await;
    }

    async fn shutdown(&self) -> JrpcResult<()> {
        Self::process_response(
            self.service
                .write()
                .await
                .call(self.create_simple_request("shutdown"))
                .await?,
        )
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let _ = self
            .service
            .write()
            .await
            .call(Self::create_notification("textDocument/didOpen", params))
            .await;
    }
}

impl<S> Service<Request> for Server<S>
where
    S: Service<Request, Response = Option<Response>, Error = ExitedError> + Send + Sync + 'static,
    <S as Service<Request>>::Future: Send,
{
    type Response = Option<Response>;
    type Error = ExitedError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let write_guard = self.service.clone().write_owned();

        Box::pin(async move {
            let mut write_guard = write_guard.await;
            write_guard.call(req).await
        })
    }
}

impl<S> Stream for Server<S> 
where
    S: Stream<Item = Request> + Unpin
{
    type Item = Request;

    fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Option<Self::Item>> {
        self.service.try_write().map(|mut guard| guard.poll_next_unpin(cx)).unwrap_or_else(|_| Poll::Pending)
    }
}


