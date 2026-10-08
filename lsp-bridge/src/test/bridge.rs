use std::{fs::File, io::Read, time::Duration};

use crate::{
    layer::{
        log::{LogLayer, LogService},
        server::{LsLayer, LsServer},
    },
    test::{DummyService, log::configure_logger},
    transport::{BaseLsService, Server, core::LsHandle},
};
use log::debug;
use rstest::{Context, fixture, rstest};
use serde_json::json;
use tower_lsp_server::{
    LanguageServer,
    ls_types::{
        ClientCapabilities, DidOpenTextDocumentParams, InitializeParams, InitializedParams,
        PublishDiagnosticsParams, TextDocumentClientCapabilities, TextDocumentItem,
        TextDocumentSyncClientCapabilities, Uri, WorkspaceClientCapabilities, WorkspaceFolder,
        notification::{Notification, PublishDiagnostics},
    },
};

#[fixture]
async fn bridge() -> (LsServer<LogService<BaseLsService>>, DummyService) {
    let build = escargot::CargoBuild::new()
        .bin("lsp-bridge")
        .into_command()
        .output()
        .unwrap();
    debug!("{build:#?}");

    let main = assert_cmd::cargo::cargo_bin("lsp-bridge");
    let (handle, task) =
        LsHandle::from_server(Server::spawn::<_, _, String>(main, vec![]).unwrap()).into_parts();
    tokio::spawn(task);
    let handle = handle
        .layer(LogLayer::new("test", "client", false))
        .layer(LsLayer::new("bridge"));

    let dummy_service = DummyService::default();
    let service = handle.connect(dummy_service.clone()).schedule();

    (service, dummy_service)
}

#[rstest]
#[tokio::test(flavor = "current_thread")]
async fn bridge_shuts_down_gracefully(
    #[context] ctx: Context,
    #[future] mut bridge: (LsServer<LogService<BaseLsService>>, DummyService),
) -> Result<(), Box<dyn std::error::Error>> {
    configure_logger(ctx);
    let (bridge, _) = bridge.await;

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

#[rstest]
#[tokio::test(flavor = "current_thread")]
async fn jdtls_finds_diagnostics(
    #[context] ctx: Context,
    #[future] mut bridge: (LsServer<LogService<BaseLsService>>, DummyService),
) -> Result<(), Box<dyn std::error::Error>> {
    configure_logger(ctx);
    let (bridge, dummy_service) = bridge.await;
    let root_uri = Uri::from_file_path(
        "/home/dennis/RustroverProjects/safe-fluent-dsls/lsp-bridge/src/testbed",
    )
    .unwrap();
    let test_file_uri =
        Uri::from_file_path(root_uri.to_file_path().unwrap().join("A.java")).unwrap();

    bridge
        .initialize(InitializeParams {
            process_id: Some(std::process::id()),
            #[allow(deprecated)]
            root_uri: Some(root_uri.clone()),
            workspace_folders: Some(vec![WorkspaceFolder {
                uri: root_uri,
                name: "testbed".into(),
            }]),
            capabilities: ClientCapabilities {
                text_document: Some(TextDocumentClientCapabilities {
                    synchronization: Some(TextDocumentSyncClientCapabilities {
                        did_save: Some(true),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                workspace: Some(WorkspaceClientCapabilities {
                    workspace_folders: Some(true),
                    configuration: Some(true),
                    ..Default::default()
                }),
                ..Default::default()
            },
            initialization_options: Some(json!({ "settings": { "java": {} } })),
            ..Default::default()
        })
        .await?;
    bridge.initialized(InitializedParams {}).await;

    let mut test_file_string = String::new();
    File::open(test_file_uri.to_file_path().unwrap())?.read_to_string(&mut test_file_string)?;
    bridge
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: test_file_uri.clone(),
                version: 0,
                language_id: "java".into(),
                text: test_file_string,
            },
        })
        .await;

    let mut diagnostics = vec![];
    while diagnostics.is_empty() {
        for notification in dummy_service
            .encountered_requests
            .entry(None)
            .or_default()
            .drain(0..)
        {
            if notification.method() == PublishDiagnostics::METHOD {
                let params: PublishDiagnosticsParams =
                    serde_json::from_value(notification.into_parts().2.unwrap())?;
                if params.uri == test_file_uri {
                    diagnostics.extend(params.diagnostics);
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    Ok(())
}
