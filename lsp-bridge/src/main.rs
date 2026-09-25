use crate::{
    bridge::Bridge,
    transport::{BaseLsHandle, LsHandleExt},
};
use tower_lsp_server::Server;

pub mod bridge;
mod codec;
mod server;
pub mod transport;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let jdtls= BaseLsHandle::spawn("jdtls", args.into_iter().skip(1))?;
    let jsonls= BaseLsHandle::spawn("vscode-json-languageserver", &["--stdio"])?;

    let (bridge, socket) = Bridge::builder()
        .main_server(jdtls)
        .embedded_server(jsonls)
        .build()
        .unwrap();
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    Server::new(stdin, stdout, socket).serve(bridge).await;

    Ok(())
}
