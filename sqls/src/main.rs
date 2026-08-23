use tower_lsp::jsonrpc::Result as RpcResult;
use tower_lsp::lsp_types::{InitializeParams, InitializeResult, InitializedParams, MessageType};
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
    client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> RpcResult<InitializeResult> {
        Ok(InitializeResult::default())
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn shutdown(&self) -> RpcResult<()> {
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let (service, client_socket) = LspService::new(|client| Backend { client: client });
    let server = tokio::spawn(Server::new(tokio::io::stdin(), tokio::io::stdout(), client_socket).serve(service));
    match server.await {
        Ok(_) => { println!("SQLS DONE") }
        Err(e) => { println!("Error during SQLS shutoff: {e}") }
    }
}
