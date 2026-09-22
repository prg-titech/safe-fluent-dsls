mod router;

use std::ffi::OsStr;
use std::process::Stdio;
use std::task::Poll;

use crate::server::{CallLanguageServer, ChildLanguageServer, LanguageServerSender, SendError};
use crate::transport::ExitedError;
use crate::transport::handle::Message;
use futures::SinkExt;
use tokio::process::Command;
use tower::Service;
use tower_lsp_server::jsonrpc::{Request, Response, Result as JrpcResult};
use tower_lsp_server::ls_types::*;
use tower_lsp_server::{Client, ClientSocket, LanguageServer, LspService};

#[derive(Default)]
struct DummyLanguageServer;

impl LanguageServer for DummyLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> JrpcResult<InitializeResult> {
        todo!()
    }

    async fn shutdown(&self) -> JrpcResult<()> {
        todo!()
    }
}

pub struct Bridge {
    client: Client,
    server: LanguageServerSender,
}

pub fn create_client() -> (Client, ClientSocket) {
    let mut result = None;
    let (_, socket) = LspService::new(|client| {
        result = Some(client);
        DummyLanguageServer
    });
    (result.unwrap(), socket)
}

impl Bridge {
    pub fn stdio<S: AsRef<OsStr>>(command: S, args: &[S]) -> (Bridge, ClientSocket) {
        let (client, socket) = create_client();
        let mut child = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let child_stdin = child.stdin.take().unwrap();
        let child_stdout = child.stdout.take().unwrap();
        let server = ChildLanguageServer::connect(child_stdout, child_stdin);
        (Bridge::connect(client, server), socket)
    }

    pub fn connect(client: Client, server: ChildLanguageServer) -> Bridge {
        let (sender, receiver) = server.split();
        let (_, mut pending_requests) = receiver.into_parts();
        let mut output_tx = sender.output_tx.clone();
        let mut _client = client.clone();
        let forward_server_requests = async move {
            while let Ok(request) = pending_requests.recv().await {
                let response = _client.call(request).await?;
                if let Some(response) = response {
                    output_tx.send(Message::Response(response)).await?;
                }
            }
            Ok::<_, SendError>(())
        };
        tokio::spawn(forward_server_requests);

        Bridge {
            client,
            server: sender,
        }
    }
}

impl Service<Request> for Bridge {
    type Response = Option<Response>;
    type Error = ExitedError;
    type Future = CallLanguageServer;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        self.server.call(req)
    }
}
