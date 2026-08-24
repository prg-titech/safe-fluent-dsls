mod lsp;
mod iter;

use lsp::Sqls;
use tower_lsp::{LspService, Server};

#[tokio::main]
async fn main() {
    let (service, client_socket) = LspService::new(Sqls::new);
    let server = tokio::spawn(Server::new(tokio::io::stdin(), tokio::io::stdout(), client_socket).serve(service));
    server.await.unwrap();
}
