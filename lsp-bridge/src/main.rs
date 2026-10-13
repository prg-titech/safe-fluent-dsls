use std::{error::Error, fs::File};

use crate::{
    bridge::{core::simple::SimpleBridge, create_bridge, java_json::server::JdtlsJsonServer},
    layer::log::LogLayer,
    transport::core::LsHandle,
};
use futures::{FutureExt, select};
use log::{LevelFilter, info};
use simplelog::{ConfigBuilder, WriteLogger};
use tower::Layer;
use tower_lsp_server::Server;

pub mod bridge;
mod codec;
pub mod layer;
pub mod service;
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

    let (bridge, mut socket) = create_bridge(&["jdtls", "jsonls"]);
    let client_socket = socket.client_socket().unwrap();
    let bridge = JdtlsJsonServer::new(bridge, socket).into_router().await;
    info!("Bridge constructed");

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    Server::new(stdin, stdout, client_socket)
        .serve(LogLayer::new("client", "bridge", false).layer(bridge))
        .await;

    Ok(())
}
