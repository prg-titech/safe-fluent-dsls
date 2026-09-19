use std::{
    borrow::Cow,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
};

use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::RwLock;
use tower::Service;
use tower_lsp_server::{
    LanguageServer,
    jsonrpc::{self, Request, Response, Result as JrpcResult},
    ls_types::*,
};

use crate::transport::ExitedError;

pub struct Server<S> {
    server_socket: Arc<RwLock<S>>,
    current_id: AtomicU32,
}

impl<S> Server<S>
where
    S: Service<Request, Response = Option<Response>, Error = ExitedError> + Send + Sync + 'static,
{
    pub fn new(socket: S) -> Self {
        Self {
            server_socket: Arc::new(RwLock::new(socket)),
            current_id: AtomicU32::new(0),
        }
    }

    pub async fn exit(&self) -> JrpcResult<()> {
        self.server_socket
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
            .server_socket
            .write()
            .await
            .call(self.create_request("initialize", params))
            .await?;
        Self::process_response(response)
    }

    async fn initialized(&self, params: InitializedParams) {
        let _ = self
            .server_socket
            .write()
            .await
            .call(Self::create_notification("initialized", params))
            .await;
    }

    async fn shutdown(&self) -> JrpcResult<()> {
        Self::process_response(
            self.server_socket
                .write()
                .await
                .call(self.create_simple_request("shutdown"))
                .await?,
        )
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let _ = self
            .server_socket
            .write()
            .await
            .call(Self::create_notification("textDocument/didOpen", params))
            .await;
    }
}
