use std::io::Read;

use tower_lsp_server::LanguageServer;
use tower_lsp_server::ls_types::{
    ClientCapabilities, DidOpenTextDocumentParams, GeneralClientCapabilities, InitializeParams,
    InitializedParams, PositionEncodingKind, TextDocumentClientCapabilities, TextDocumentItem,
    TextDocumentSyncClientCapabilities, Uri,
};

use crate::{server::Server, transport::LanguageServerService};

mod codec;
mod server;
pub mod transport;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (sqls_service, mut sqls_requests) =
        LanguageServerService::stdio("vscode-json-languageserver", &["--stdio"])?;
    let sqls = Server::new(sqls_service);

    let join_request_handler = tokio::spawn(async move {
        while let Ok(request) = sqls_requests.recv().await {
            let request_str = serde_json::to_string(&request).unwrap();
            if request.id().is_some() {
                println!("Received Request from Sqls: {}", request_str);
            } else {
                println!("Received Notification from Sqls: {}", request_str);
            }
        }
    });

    let server_capabilities = sqls
        .initialize(InitializeParams {
            process_id: Some(std::process::id()),
            capabilities: ClientCapabilities {
                general: Some(GeneralClientCapabilities {
                    position_encodings: Some(vec![PositionEncodingKind::UTF8]),
                    ..Default::default()
                }),
                text_document: Some(TextDocumentClientCapabilities {
                    synchronization: Some(TextDocumentSyncClientCapabilities {
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        })
        .await?;
    println!(
        "SQLS capabilities: {}",
        serde_json::to_string_pretty(&server_capabilities)?
    );

    sqls.initialized(InitializedParams {}).await;

    let test_uri =
        Uri::from_file_path(std::env::current_dir()?.join("testbed/file1.json")).unwrap();
    let mut text = String::new();
    let _ = std::fs::File::open(test_uri.to_file_path().unwrap())?.read_to_string(&mut text)?;
    sqls.did_open(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: test_uri,
            language_id: "json".into(),
            version: 0,
            text: text,
        },
    })
    .await;

    sqls.shutdown().await?;

    sqls.exit().await?;

    join_request_handler.await?;
    Ok(())
}
