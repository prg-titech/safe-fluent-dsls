use std::io::Read;
use tower_lsp_server::LanguageServer;
use tower_lsp_server::ls_types::{
    ClientCapabilities, DidOpenTextDocumentParams, GeneralClientCapabilities, InitializeParams,
    InitializedParams, PositionEncodingKind, TextDocumentClientCapabilities, TextDocumentItem,
    TextDocumentSyncClientCapabilities, Uri,
};
use futures::StreamExt;
use crate::server::Server;
use crate::{transport::LanguageServerService};

mod codec;
mod server;
pub mod transport;
pub mod bridge;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json_service = LanguageServerService::stdio("vscode-json-languageserver", &["--stdio"])?;
    let mut json_ls = Server::new(json_service);

    let server_capabilities = json_ls
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

    json_ls.initialized(InitializedParams {}).await;

    println!("initialized");

    let test_uri =
        Uri::from_file_path(std::env::current_dir()?.join("testbed/file1.json")).unwrap();
    let mut text = String::new();
    let _ = std::fs::File::open(test_uri.to_file_path().unwrap())?.read_to_string(&mut text)?;
    json_ls.did_open(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: test_uri,
            language_id: "json".into(),
            version: 0,
            text: text,
        },
    })
    .await;

    println!("file opened");

    let request = json_ls.next().await.unwrap();
    println!("{}", serde_json::to_string(&request).unwrap());

    json_ls.shutdown().await?;

    println!("shutdown signal send");

    json_ls.exit().await?;

    Ok(())
}
