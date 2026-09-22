use crate::bridge::Bridge;
use tower_lsp_server::Server;

pub mod bridge;
mod codec;
mod server;
pub mod transport;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (bridge, socket) = Bridge::stdio("vscode-json-languageserver", &["--stdio"]);
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    Server::new(stdin, stdout, socket).serve(bridge).await;

    Ok(())
}
