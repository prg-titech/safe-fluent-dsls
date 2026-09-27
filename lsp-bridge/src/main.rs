use std::{error::Error, fs::File};

use crate::{
    bridge::BridgeBuilder, logging::LogLayer, transport::{BaseLsHandle, LsHandleExt},
};
use log::{LevelFilter, info};
use simplelog::{ConfigBuilder, WriteLogger};
use tower_lsp_server::Server;

pub mod bridge;
mod codec;
mod logging;
mod server;
pub mod transport;

#[cfg(test)]
mod test;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let logger_config = ConfigBuilder::new().build();
    let _log_init: Result<(), _> = std::env::current_exe()
        .and_then(|p| p.parent().and_then(|p| p.parent()).map(|p| p.to_path_buf()).ok_or(std::io::Error::last_os_error()))
        .and_then(|p| File::create(p.join("lsp-bridge.log"))).map_err(|err| Box::new(err) as Box<dyn Error>)
        .and_then(|f| WriteLogger::init(LevelFilter::Trace, logger_config, f).map_err(|e| e.into()));

    info!("Constructing bridge...");

    let args: Vec<String> = std::env::args().collect();
    let jdtls = BaseLsHandle::spawn("jdtls", args.iter().skip(1))?;
    info!("Connected to jdtls with args: {}", args.join(" "));

    let jsonls = BaseLsHandle::spawn("vscode-json-languageserver", &["--stdio"])?;
    info!("Connected to jsonls");

    let (bridge, socket) = BridgeBuilder::default()
        .main_server(jdtls)
        .embedded_server(jsonls)
        .log_main_server(LogLayer::new("bridge", "jdtls", false))
        .log_embedded_server(LogLayer::new("bridge", "jsonls", false))
        .log_client(LogLayer::new("client", "bridge", true))
        .build();
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    info!("Bridge constructed");
    Server::new(stdin, stdout, socket).serve(bridge).await;

    Ok(())
}
