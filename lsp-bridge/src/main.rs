use async_lsp_client::{LspServer, ServerMessage};
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::error::TryRecvError;
use tower_lsp::jsonrpc::{self};
use tower_lsp::{lsp_types::*};

#[tokio::main(flavor="multi_thread")]
async fn main() -> Result<(), jsonrpc::Error> {
    let (server, rx) = LspServer::new("../sqls/target/debug/sqls", []);
    
    let handle = tokio::spawn(message_loop(rx));

    let initialize_result = server.initialize(InitializeParams::default()).await?;
    println!("{initialize_result:#?}");

    server.initialized().await;
    println!("initialized");

    handle.await.expect("Error occurred on shutdown");
    Ok(())
}

async fn message_loop(mut rx: Receiver<ServerMessage>) {
    let mut stdout = tokio::io::stdout();
    loop {
        let msg = rx.try_recv();
        match msg {
            Err(TryRecvError::Disconnected) => {break}
            Err(TryRecvError::Empty) => {
                // stdout.write_all(format!("EMPTY {counter}\n").as_bytes()).await.expect("UNABLE TO WRITE TO STDOUT");
            }
            Ok(ServerMessage::Notification(msg)) => {
                match msg.method.as_str() {
                    "window/logMessage" => {
                        let params: LogMessageParams = serde_json::from_value(msg.params.expect("Missing parameters")).expect("Invalid parameters");
                        stdout.write_all(format!("{}\n", params.message).as_bytes()).await.expect("UNABLE TO WRITE TO STDOUT");
                    }
                    _ => {
                        todo!("notification {} not implemented", msg.method)
                    }
                }
            }
            Ok(ServerMessage::Request(msg)) => {
                todo!("server -> client request '{}' not implemented", msg.method())
            }
        }
    }
} 
