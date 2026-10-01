use std::{error::Error, fs::File};

use crate::{bridge::BridgeBuilder, logging::LogLayer};
use log::{LevelFilter, info};
use simplelog::{ConfigBuilder, WriteLogger};
use tower::Layer;
use tower_lsp_server::Server;

pub mod bridge;
mod codec;
mod id_layer;
mod logging;
mod server;
pub mod transport;

#[cfg(test)]
mod test;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let logger_config = ConfigBuilder::new().build();
    let _log_init: Result<(), _> = std::env::current_exe()
        .and_then(|p| {
            p.parent()
                .and_then(|p| p.parent())
                .map(|p| p.to_path_buf())
                .ok_or(std::io::Error::last_os_error())
        })
        .and_then(|p| File::create(p.join("lsp-bridge.log")))
        .map_err(|err| Box::new(err) as Box<dyn Error>)
        .and_then(|f| {
            WriteLogger::init(LevelFilter::Trace, logger_config, f).map_err(|e| e.into())
        });

    info!("Constructing bridge...");

    let args: Vec<String> = std::env::args().collect();
    let bridge_task = BridgeBuilder::default()
        .main_server("jdtls", "jdtls", args.iter().skip(1))
        .log_main_server()
        .log_main_client()
        .embedded_server("jsonls", "vscode-json-languageserver", &["--stdio"])
        .log_embedded_server()
        .log_embedded_client()
        .build();
    let (bridge, socket) = bridge_task.schedule();
    let bridge = LogLayer::new("client", "bridge", false).layer(bridge);
    info!("Bridge constructed");

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    Server::new(stdin, stdout, socket).serve(bridge).await;

    Ok(())
}
