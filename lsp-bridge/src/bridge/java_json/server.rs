use std::error::Error;
use std::fmt::Display;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use std::{io, mem};

use futures::Sink;
use futures::join;
use log::info;
use tokio::io::AsyncBufReadExt;
use tokio::process::{Child, Command};
use tokio::sync::RwLock;
use tower::layer::util::Identity;
use tower::{Layer, Service};
use tower_lsp_server::jsonrpc::{Request, Response};
use tower_lsp_server::ls_types::request::{Initialize, Request as LsRequest, Shutdown};
use tower_lsp_server::ls_types::{InitializeParams, InitializeResult, Uri};
use tower_lsp_server::{LanguageServer, Loopback, Server, jsonrpc};

use crate::bridge::java_json::java_client::{JdtlsClient, LanguageStatus};
use crate::bridge::router::{DefaultingRouter, Router};
use crate::bridge::{Bridge, BridgeSocket};
use crate::layer::log::LogLayer;
use crate::layer::server::LsServer;
use crate::transport::error::ExitedError;

pub enum JdtlsJsonServerInner<S, L> {
    Created(Bridge<S>, BridgeSocket<L>),
    Initialized(Bridge<S>, JdtlsClient<S>, [Child; 2]),
    Shutdown,
}

impl<S, L> Default for JdtlsJsonServerInner<S, L> {
    fn default() -> Self {
        Self::Shutdown
    }
}

impl<S, L> JdtlsJsonServerInner<S, L> {
    fn map<F>(&mut self, fun: F)
    where
        F: FnOnce(Self) -> Self,
    {
        let value = mem::take(self);
        *self = fun(value)
    }

    fn socket(&mut self) -> &mut BridgeSocket<L> {
        match self {
            Self::Created(_, socket) => socket,
            _ => panic!("Socket is not available! Aborting..."),
        }
    }

    fn bridge(&self) -> Bridge<S> {
        match self {
            Self::Created(bridge, _) => bridge.clone(),
            Self::Initialized(bridge, _, _) => bridge.clone(),
            _ => panic!("Jdtls-Jsonls Bridge has not been initialized! Aborting..."),
        }
    }

    fn children(&mut self) -> Option<&mut [Child]> {
        match self {
            Self::Initialized(_, _, children) => Some(children),
            _ => None,
        }
    }

    fn jdtls_client(&self) -> JdtlsClient<S>
    where
        S: Clone,
    {
        match self {
            Self::Initialized(_, client, _) => client.clone(),
            _ => panic!("Jdtls-Jsonls Bridge has not been initialized! Aborting..."),
        }
    }
}

#[derive(Clone)]
pub struct JdtlsJsonServer<S, L> {
    inner: Arc<RwLock<JdtlsJsonServerInner<S, L>>>,
}

impl<S, L> JdtlsJsonServer<S, L>
where
    S: Clone,
{
    pub fn new(bridge: Bridge<S>, socket: BridgeSocket<L>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(JdtlsJsonServerInner::Created(bridge, socket))),
        }
    }

    pub async fn into_router(self) -> DefaultingRouter<Self, LsServer<S>, ExitedError>
    where
        S: Service<Request, Response = Option<Response>> + Send + Sync + Clone + 'static,
        S::Error: Into<Box<dyn std::error::Error + Send + Sync>>
            + Into<ExitedError>
            + Display
            + Into<jsonrpc::Error>
            + Send,
        S::Future: Send + 'static,
        L: Loopback + Send + Sync + 'static,
        L::ResponseSink: Send,
        L::RequestStream: Send,
        <L::ResponseSink as Sink<Response>>::Error: Error,
    {
        let default = self.inner.read().await.bridge().server("jdtls").unwrap();
        let mut router = Router::new(self);
        let default_layer = Identity::new();
        router
            .method(Initialize::METHOD, Self::initialize, default_layer.clone())
            .method(Shutdown::METHOD, Self::shutdown, default_layer);
        DefaultingRouter::new(router, default)
    }

    async fn start_jdtls(&self, workspace_uri: &Uri) -> Result<(JdtlsClient<S>, Child), io::Error>
    where
        S: Service<Request, Response = Option<Response>> + Send + Sync + 'static,
        S::Future: Send + 'static,
        S::Error:
            Into<Box<dyn std::error::Error + Send + Sync>> + Into<ExitedError> + Display + Send,
        L: Loopback + Send + 'static,
        L::ResponseSink: Send,
        L::RequestStream: Send,
        <L::ResponseSink as Sink<Response>>::Error: Error,
    {
        let socket = self
            .inner
            .write()
            .await
            .socket()
            .server_socket("jdtls")
            .expect("Unable to get Jdtls Socket");
        let workspace_name = workspace_uri
            .to_file_path()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let workspace_hash = {
            let mut state = DefaultHasher::new();
            workspace_uri.hash(&mut state);
            format!("{:x}", state.finish())
        };
        let config_directory = std::env::home_dir().unwrap().join(".cache/jdtls/config");
        let data_directory = std::env::home_dir().unwrap().join(format!(
            ".cache/jdtls/workspace/{}_{}",
            workspace_name, workspace_hash
        ));

        let mut child = Command::new("jdtls")
            .args([
                "-configuration".as_ref(),
                config_directory.as_os_str(),
                "-data".as_ref(),
                data_directory.as_os_str(),
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let input = child.stdout.take().unwrap();
        let output = child.stdin.take().unwrap();
        let mut diagnostics = tokio::io::BufReader::new(child.stderr.take().unwrap()).lines();
        let forward_diagnostics_to_log = async move {
            while let Ok(Some(l)) = diagnostics.next_line().await {
                info!("jdtls - {l}")
            }
        };

        let server = Server::new(input, output, socket);
        let jdtls_client = JdtlsClient::new(self.client().await);
        let serve_jdtls_client = server.serve(
            LogLayer::new("client", "jdtls", true).layer(jdtls_client.clone().into_router()),
        );
        tokio::spawn(async move { join!(forward_diagnostics_to_log, serve_jdtls_client) });

        Ok((jdtls_client, child))
    }

    async fn start_jsonls(&self) -> Result<Child, io::Error>
    where
        L: Loopback + Send + 'static,
        L::RequestStream: Send,
        L::ResponseSink: Send,
        <L::ResponseSink as Sink<Response>>::Error: Error,
        S: Service<Request, Response = Option<Response>> + Send + Sync + 'static,
        S::Future: Send + 'static,
        S::Error: Into<Box<dyn Error + Send + Sync>> + Display + Send + 'static,
    {
        let mut child = Command::new("vscode-json-languageserver")
            .args(["--stdio"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let input = child.stdout.take().unwrap();
        let output = child.stdin.take().unwrap();
        let socket = self
            .inner
            .write()
            .await
            .socket()
            .server_socket("jsonls")
            .expect("Unable to get Jsonls Socket");
        let server = Server::new(input, output, socket);
        let serve_jsonls =
            server.serve(LogLayer::new("client", "jsonls", true).layer(self.client().await));

        tokio::spawn(serve_jsonls);

        Ok(child)
    }

    async fn client(&self) -> LsServer<S> {
        self.inner.read().await.bridge().client()
    }

    async fn jdtls_client(&self) -> JdtlsClient<S> {
        self.inner.read().await.jdtls_client()
    }

    async fn jdtls(&self) -> LsServer<S> {
        self.inner.read().await.bridge().server("jdtls").unwrap()
    }

    async fn jsonls(&self) -> LsServer<S> {
        self.inner.read().await.bridge().server("jsonls").unwrap()
    }
}

impl<S, L> LanguageServer for JdtlsJsonServer<S, L>
where
    S: Service<Request, Response = Option<Response>> + Send + Sync + Clone + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>
        + Into<ExitedError>
        + Display
        + Into<jsonrpc::Error>
        + Send,
    S::Future: Send + 'static,
    L: Loopback + Send + Sync + 'static,
    L::ResponseSink: Send,
    L::RequestStream: Send,
    <L::ResponseSink as Sink<Response>>::Error: Error,
{
    async fn initialize(&self, params: InitializeParams) -> jsonrpc::Result<InitializeResult> {
        let workspace_uri = &params
            .workspace_folders
            .as_ref()
            .unwrap()
            .first()
            .unwrap()
            .uri;
        if let (Ok((jdtls_client, jdtls_child)), Ok(jsonls_child)) =
            join!(self.start_jdtls(&workspace_uri), self.start_jsonls())
        {
            self.inner
                .clone()
                .write_owned()
                .await
                .map(|state| match state {
                    JdtlsJsonServerInner::Created(bridge, _) => JdtlsJsonServerInner::Initialized(
                        bridge,
                        jdtls_client,
                        [jdtls_child, jsonls_child],
                    ),
                    _ => JdtlsJsonServerInner::Shutdown,
                });
            let _ = self.jsonls().await.initialize(params.clone()).await;
            let result = self.jdtls().await.initialize(params).await?;
            while self.jdtls_client().await.get_status().await < LanguageStatus::Started {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            Ok(result)
        } else {
            Err(jsonrpc::Error::internal_error())
        }
    }

    async fn shutdown(&self) -> jsonrpc::Result<()> {
        let jdtls = self.jdtls().await;
        let jsonls = self.jsonls().await;
        let _ = join!(jdtls.shutdown(), jsonls.shutdown());

        self.inner.write().await.map(|mut inner| {
            if let Some(children) = inner.children() {
                for child in children {
                    let _ = child.kill();
                }
            }
            JdtlsJsonServerInner::Shutdown
        });
        Ok(())
    }
}
