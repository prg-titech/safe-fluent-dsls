use crate::tests::test_client::{DefaultTestClient, stdio};
use rstest::{fixture, rstest};
use tower_lsp_server::{jsonrpc::{Id, Request}, ls_types::{self, ClientCapabilities, DidOpenTextDocumentParams, InitializeParams, PublishDiagnosticsParams, TextDocumentItem, Uri, WorkspaceFolder}};

async fn read_file_from_uri(uri: &Uri) -> Result<String, tokio::io::Error> {
    tokio::fs::read_to_string(uri.to_file_path().unwrap()).await
}

#[fixture]
async fn stdio_client() -> DefaultTestClient {
    let testbed_dir = std::env::current_dir().unwrap().join("src/testbed");
    let testbed_dir = Uri::from_file_path(testbed_dir).unwrap();
    let mut client = stdio().await;

    println!("started server!");

    let initialize_params: InitializeParams = InitializeParams { 
        capabilities: ClientCapabilities::default(), 
        workspace_folders: Some(
            vec![WorkspaceFolder {name: "testbed".to_string(), uri: testbed_dir.clone() }]
        ), 
        trace: Some(ls_types::TraceValue::Off),
        ..Default::default()
    };

    let initialize_response = client
        .make_request(
            Request::build("initialize")
                .id(1)
                .params(serde_json::to_value(initialize_params).unwrap())
                .finish(),
        )
        .await;
    assert!(initialize_response.is_ok());

    client
        .send_notification(Request::build("initialized").finish())
        .await;
    client
}

#[rstest]
#[tokio::test(flavor="current_thread")]
async fn server_sends_diagnostics_on_file_event(
    #[future] mut stdio_client: DefaultTestClient
) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdio_client = stdio_client.await;

    let testbed_path = std::env::current_dir().unwrap().join("src/testbed");
    let file1_uri = Uri::from_file_path(testbed_path.join("file1.sql")).unwrap();
    let file2_uri = Uri::from_file_path(testbed_path.join("file2.sql")).unwrap();
    let file3_uri = Uri::from_file_path(testbed_path.join("nested/file3.sql")).unwrap();

    stdio_client.send_notification(
        Request::build("textDocument/didOpen")
            .params(serde_json::to_value(DidOpenTextDocumentParams { 
                text_document: TextDocumentItem { 
                    uri: file1_uri.clone(), 
                    language_id: "sql".to_string(), 
                    version: 1, 
                    text: read_file_from_uri(&file1_uri).await?
                } 
            })?)
            .finish()
    ).await;

    let publish_diagnostics = stdio_client.receive_request().await;
    assert_eq!(publish_diagnostics.id(), None);
    assert_eq!(publish_diagnostics.method(), "textDocument/publishDiagnostics");
    let params: PublishDiagnosticsParams = serde_json::from_value(publish_diagnostics.into_parts().2.unwrap())?;
    assert_eq!(params.uri, file1_uri);
    assert_eq!(params.version, None);
    //assert_eq!(params.diagnostics, vec![]);

    Ok(())
}

#[rstest]
#[tokio::test(flavor="current_thread")]
async fn server_shuts_down_cleanly(
    #[future] mut stdio_client: DefaultTestClient,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdio_client = stdio_client.await;

    let shutdown_response = stdio_client
        .make_request(Request::build("shutdown").id(99).finish())
        .await;

    assert!(shutdown_response.is_ok());
    assert_eq!(shutdown_response.id().clone(), Id::Number(99));

    stdio_client
        .send_notification(Request::build("exit").finish())
        .await;

    let exit_status = stdio_client.child.wait().await.unwrap();

    assert!(exit_status.success());
    Ok(())
}
