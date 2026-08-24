use std::process;
use std::time::Duration;

use async_lsp_client::{LspServer, ServerMessage};
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::error::TryRecvError;
use tower_lsp::jsonrpc::{self};
use tower_lsp::lsp_types::notification::{DidChangeTextDocument, DidOpenTextDocument};
use tower_lsp::lsp_types::*;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), jsonrpc::Error> {
    let (server, rx) = LspServer::new("cargo", ["run", "--manifest-path", "../sqls/Cargo.toml"]);

    let handle = tokio::spawn(message_loop(rx));

    let initialize_result = server
        .initialize(InitializeParams {
            process_id: Some(process::id()),
            capabilities: ClientCapabilities {
                text_document: Some(TextDocumentClientCapabilities {
                    semantic_tokens: Some(SemanticTokensClientCapabilities {
                        requests: SemanticTokensClientCapabilitiesRequests {
                            range: Some(false),
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                        },
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        })
        .await?;
    println!("{initialize_result:#?}");

    server.initialized().await;
    println!("initialized");

    let example_document = TextDocumentItem {
        uri: Url::parse("file://select_children.sql").unwrap(),
        language_id: "sql".to_owned(),
        version: 1,
        text: "SELECT children FROM Students".to_owned(),
    };
    let example_id = TextDocumentIdentifier {
        uri: example_document.uri.clone(),
    };

    server
        .send_notification::<DidOpenTextDocument>(DidOpenTextDocumentParams {
            text_document: example_document.clone(),
        })
        .await;

    server
        .send_notification::<DidChangeTextDocument>(DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                uri: example_id.uri,
                version: 2,
            },
            content_changes: vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 0,
                        character: 28,
                    },
                    end: Position {
                        line: 0,
                        character: 28,
                    },
                }),
                range_length: None,
                text: " WHERE children = \"10\"".to_owned(),
            }],
        })
        .await;

    tokio::time::timeout(Duration::from_secs(1), handle)
        .await
        .expect_err("Handler crashed");
    server.shutdown().await?;
    server.exit().await;

    /*let tokens: SemanticTokensResult = server
        .send_request::<SemanticTokensFullRequest>(SemanticTokensParams {
            text_document: example_id,
            work_done_progress_params: WorkDoneProgressParams {
                work_done_token: None,
            },
            partial_result_params: PartialResultParams {
                partial_result_token: None,
            },
        })
        .await
        .expect("Error computing semantic tokens")
        .unwrap();

    println!("{tokens:?}");*/

    Ok(())
}

async fn message_loop(mut rx: Receiver<ServerMessage>) {
    let mut stdout = tokio::io::stdout();
    loop {
        let msg = rx.try_recv();
        match msg {
            Err(TryRecvError::Disconnected) => break,
            Err(TryRecvError::Empty) => {
                // stdout.write_all(format!("EMPTY {counter}\n").as_bytes()).await.expect("UNABLE TO WRITE TO STDOUT");
            }
            Ok(ServerMessage::Notification(msg)) => match msg.method.as_str() {
                "window/logMessage" => {
                    let params: LogMessageParams =
                        serde_json::from_value(msg.params.expect("Missing parameters"))
                            .expect("Invalid parameters");
                    stdout
                        .write_all(format!("{}\n", params.message).as_bytes())
                        .await
                        .expect("UNABLE TO WRITE TO STDOUT");
                }
                "textDocument/publishDiagnostics" => {
                    let params: PublishDiagnosticsParams =
                        serde_json::from_value(msg.params.expect("Missing parameters").clone())
                            .expect("Invalid parameters");
                    for diagnostic in params.diagnostics {
                        stdout
                            .write_all(format!("PARSE ERROR: {:#?}\n", diagnostic.range).as_bytes())
                            .await
                            .unwrap();
                    }
                }
                _ => {
                    todo!("notification {} not implemented", msg.method)
                }
            },
            Ok(ServerMessage::Request(msg)) => match msg.method() {
                _ => {
                    todo!("result {} not implemented", msg.method())
                }
            },
        }
    }
}
