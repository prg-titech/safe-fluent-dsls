use std::{error::Error, fs::File};

use crate::{
    bridge::{Bridge, BridgeBuilder},
    logging::{LogLayer, LogService},
    server::{LsLayer, LsServer},
    transport::{
        BaseLsService, Server,
        core::{LsHandle, LsSocket},
        error::ExitedError,
    },
};
use futures::{SinkExt, StreamExt};
use log::{LevelFilter, info};
use rstest::{fixture, rstest};
use serde_json::json;
use simplelog::{ConfigBuilder, WriteLogger};
use tower::{Layer, util::BoxService};
use tower_lsp_server::{
    LanguageServer,
    jsonrpc::{Request, Response},
    ls_types::{InitializeParams, InitializedParams},
};

#[fixture]
async fn bridge() -> LsServer<LogService<BaseLsService>> {
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

    let main = assert_cmd::cargo::cargo_bin("lsp-bridge");
    let (handle, task) =
        LsHandle::from_server(Server::spawn::<_, _, String>(main, vec![]).unwrap()).into_parts();
    tokio::spawn(task);
    let handle = handle
        .layer(LogLayer::new("test", "client", false))
        .layer(LsLayer);
    let (service, socket) = handle.split();
    tokio::spawn(async move {
        let (mut rx, mut tx) = socket.split();
        while let Some(r) = rx.next().await {
            info!("Got request: {}", serde_json::to_string(&r).unwrap());
            if r.id().is_some() {
                let dummy_response = Response::from_ok(r.id().cloned().unwrap(), json!({}));
                info!(
                    "Send dummy response: {}",
                    serde_json::to_string(&dummy_response).unwrap()
                );
                let _ = tx.send(dummy_response).await;
            }
        }
    });

    service
}

#[rstest]
#[tokio::test(flavor = "current_thread")]
async fn jdtls_shuts_down_gracefully() -> Result<(), Box<dyn Error>> {
    let logger_config = ConfigBuilder::new().build();
    let _log_init: Result<(), _> = std::env::current_exe()
        .and_then(|p| {
            p.parent()
                .and_then(|p| p.parent())
                .and_then(|p| p.parent())
                .map(|p| p.to_path_buf())
                .ok_or(std::io::Error::last_os_error())
        })
        .and_then(|p| File::create(p.join("lsp-bridge.log")))
        .map_err(|err| Box::new(err) as Box<dyn Error>)
        .and_then(|f| {
            WriteLogger::init(LevelFilter::Trace, logger_config, f).map_err(|e| e.into())
        });

    let jdtls_server = Server::spawn::<_, _, String>("jdtls", vec![])?;
    let (handle, socket) = LsHandle::from_server(jdtls_server)
        .schedule()
        .layer(LogLayer::new("test", "jdtls", false))
        .layer(LsLayer)
        .split();

    let (mut rx, mut tx) = socket.split();
    tokio::spawn(async move {
        while let Some(request) = rx.next().await {
            info!(
                "Request:  test <-- jdtls {}",
                serde_json::to_string(&request).unwrap()
            );
            if request.id().is_some() {
                let dummy_response = Response::from_ok(request.into_parts().1.unwrap(), json!({}));
                info!(
                    "Response: test --> jdtls {}",
                    serde_json::to_string(&dummy_response).unwrap()
                );
                let _ = tx.send(dummy_response).await;
            }
        }
    });

    handle.initialize(InitializeParams::default()).await?;
    handle.initialized(InitializedParams {}).await;
    handle.shutdown().await?;
    handle.exit().await;

    Ok(())
}

#[rstest]
#[tokio::test(flavor = "current_thread")]
async fn bridge_shuts_down_gracefully(
    #[future] mut bridge: LsServer<LogService<BaseLsService>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let bridge = bridge.await;

    bridge
        .initialize(InitializeParams {
            process_id: Some(std::process::id()),
            ..Default::default()
        })
        .await?;

    bridge.initialized(InitializedParams {}).await;

    bridge.shutdown().await?;

    bridge.exit().await;
    Ok(())
}
