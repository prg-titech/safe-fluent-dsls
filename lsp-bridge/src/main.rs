use std::{error::Error, fs::File, time::Duration};

use crate::{bridge::core::simple::SimpleBridge, layer::log::LogLayer, transport::core::LsHandle};
use futures::{FutureExt, select};
use log::{LevelFilter, debug, info};
use simplelog::{ConfigBuilder, WriteLogger};
use tower_lsp_server::Server;

pub mod bridge;
mod codec;
pub mod layer;
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

    let jdtls = LsHandle::from_server(transport::Server::spawn(
        "jdtls",
        [
            "-configuration",
            "/home/dennis/.cache/jdtls/config",
            "-data",
            "/home/dennis/.cache/jdtls/workspace/1234abcd9876fedc",
        ],
    )?)
    .schedule()
    .layer(LogLayer::new("client", "jdtls", false));
    let (bridge, socket) =
        SimpleBridge::new(move |client| SimpleBridge::connect_with_log(client, jdtls, "jdtls"));
    info!("Bridge constructed");

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    /*let mut healthcheck = async {
        let mut current_duration = Duration::from_millis(5);
        let max_duration = Duration::from_secs(5);
        loop {
            tokio::time::sleep(current_duration).await;
            debug!("Bridge is healthy");
            current_duration = Duration::min(current_duration * 2, max_duration);
        }
    }
    .boxed()
    .fuse();*/
    let mut serve = Server::new(stdin, stdout, socket)
        .serve(bridge)
        .boxed()
        .fuse();
    select! {
        // _ = healthcheck => (),
        _ = serve => ()
    };

    Ok(())
}
