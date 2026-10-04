use crate::{
    logging::{LogLayer, LogService},
    server::{LsLayer, LsServer},
    test::log::configure_logger,
    transport::{BaseLsService, Server, core::LsHandle, error::ExitedError},
};
use log::info;
use rstest::{Context, fixture, rstest};
use serde_json::json;
use tower_lsp_server::{
    LanguageServer,
    jsonrpc::{Request, Response},
    ls_types::{InitializeParams, InitializedParams},
};

#[fixture]
async fn bridge() -> LsServer<LogService<BaseLsService>> {
    let main = assert_cmd::cargo::cargo_bin("lsp-bridge");
    let (handle, task) =
        LsHandle::from_server(Server::spawn::<_, _, String>(main, vec![]).unwrap()).into_parts();
    tokio::spawn(task);
    let handle = handle
        .layer(LogLayer::new("test", "client", false))
        .layer(LsLayer);
    let dummy_service = tower::service_fn(async |request: Request| {
        info!("Got request: {}", serde_json::to_string(&request).unwrap());
        if request.id().is_some() {
            let dummy_response = Response::from_ok(request.id().cloned().unwrap(), json!({}));
            info!(
                "Send dummy response: {}",
                serde_json::to_string(&dummy_response).unwrap()
            );
            Ok::<_, ExitedError>(Some(dummy_response))
        } else {
            Ok(None)
        }
    });
    let service = handle.connect(dummy_service).schedule();

    service
}

#[rstest]
#[tokio::test(flavor = "current_thread")]
async fn bridge_shuts_down_gracefully(
    #[context] ctx: Context,
    #[future] mut bridge: LsServer<LogService<BaseLsService>>,
) -> Result<(), Box<dyn std::error::Error>> {
    configure_logger(ctx);
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
