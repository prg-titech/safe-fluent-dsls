use serde::{Deserialize, Serialize};
use tower_lsp_server::jsonrpc::Request;
use std::time::Duration;
use std::{path::PathBuf, process::Stdio, sync::Arc};

use assert_cmd::cargo::cargo_bin;
use escargot::CargoBuild;
use tokio::{
    io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    process::{Child, Command},
    sync::mpsc::channel,
};

use crate::requests::GetWorkspaceFiles;
use rstest::{fixture, rstest};

pub static CLIENT_CAPABILITIES: &str = include_str!("client.json");

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse<T> {
    jsonrpc: String,
    pub id: u64,
    pub result: T,
}

pub struct TestServer {
    writer_tx: tokio::sync::mpsc::Sender<String>,
    notify_rx: tokio::sync::mpsc::Receiver<()>,
    pub responses: Arc<tokio::sync::RwLock<Vec<String>>>,
    pub child: Child,
}

impl TestServer {
    fn build_binary(curr_dir: &PathBuf) -> Result<(), std::io::Error> {
        let result = CargoBuild::new()
            .bin("sqls")
            .run()
            .expect("Failed to build server");

        result.command().current_dir(curr_dir).spawn()?;
        Ok(())
    }

    fn spawn_binary(curr_dir: &PathBuf) -> Result<Child, std::io::Error> {
        let bin = cargo_bin("sqls");

        Command::new(bin)
            .current_dir(curr_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }

    pub async fn stdio() -> Result<Self, std::io::Error> {
        let curr_dir = std::env::current_dir()?;

        TestServer::build_binary(&curr_dir)?;
        let mut child = TestServer::spawn_binary(&curr_dir)?;

        let mut stdin = child.stdin.take().expect("Failed to open stdin");
        let stdout = child.stdout.take().expect("Failed to open stdout");
        let mut stdout = BufReader::new(stdout);

        let responses = Arc::new(tokio::sync::RwLock::new(vec![]));
        let responses_clone = responses.clone();

        let (notify_tx, notify_rx) = tokio::sync::mpsc::channel::<()>(100);
        let (writer_tx, mut rx) = channel::<String>(1);

        // Read messages from the server
        tokio::task::spawn(async move {
            while let Ok(message) = TestServer::read_message(&mut stdout).await {
                println!("Received Response:\n{message}");
                responses_clone.write().await.push(message);
                let _ = notify_tx.send(()).await;
            }
        });

        // Write messages to the server
        tokio::task::spawn(async move {
            while let Some(message) = rx.recv().await {
                let msg = format!("Content-Length: {}\r\n\r\n{}", message.len(), message);
                println!("Sending Request:\n{message}");
                stdin.write_all(msg.as_bytes()).await?;
                stdin.flush().await?;
            }
            Ok::<(), std::io::Error>(())
        });

        Ok(Self {
            notify_rx,
            writer_tx,
            responses,
            child,
        })
    }

    /// Waits until `n` messages have been received.
    pub async fn wait_for_messages(&mut self, n: usize) {
        for _ in 0..n {
            self.notify_rx.recv().await;
        }
    }

    async fn read_message<T: AsyncBufRead + std::marker::Unpin>(
        stdout: &mut T,
    ) -> Result<String, std::io::Error> {
        let mut headers = String::new();
        loop {
            let mut line = String::new();
            stdout.read_line(&mut line).await?;
            if line == "\r\n" {
                break; // End of headers
            }
            headers.push_str(&line);
        }

        // Extract Content-Length
        let content_length = headers
            .lines()
            .find_map(|line| {
                if line.to_lowercase().starts_with("content-length:") {
                    line["Content-Length:".len()..].trim().parse::<usize>().ok()
                } else {
                    None
                }
            })
            .unwrap_or(0);

        // Read full message body
        let mut body = vec![0; content_length];
        stdout.read_exact(&mut body).await?;

        Ok(String::from_utf8_lossy(&body).to_string())
    }

    pub async fn write_message(
        &mut self,
        message: &str,
    ) -> Result<(), tokio::sync::mpsc::error::SendError<std::string::String>> {
        self.writer_tx.send(message.to_string()).await
    }
}

#[fixture]
async fn stdio_server() -> TestServer {
    let testbed_dir = std::env::current_dir().unwrap().join("src/testbed");
    let mut server = TestServer::stdio().await.unwrap();

    // Send client capabilities
    // Workspace folder URI is prefixed with current directory
    server
        .write_message(
            CLIENT_CAPABILITIES
                .replace(
                    "file:///testbed",
                    format!("file:///{}", testbed_dir.to_str().unwrap()).as_str(),
                )
                .as_str(),
        )
        .await
        .unwrap();

    // Initialized
    server
        .write_message(r#"{"jsonrpc": "2.0", "method": "initialized", "params": {}}"#)
        .await
        .unwrap();

    server.wait_for_messages(1).await;

    // Request with id 1 is the initialized response
    assert!(server.responses.read().await[0].contains(r#""id":1"#));
    server.responses.write().await.clear();

    server
}

#[rstest]
#[tokio::test]
async fn workspace_files(
    #[future] mut stdio_server: TestServer,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdio_server = stdio_server.await;

    println!("Hello?");

    // Workspace files are checked on initialization
    // We expect all files from the testbed directory to be present

    stdio_server
        .write_message(&GetWorkspaceFiles::request(2))
        .await?;

    stdio_server.wait_for_messages(1).await;

    let responses = stdio_server.responses.read().await;

    let workspace_response: JsonRpcResponse<Vec<String>> = serde_json::from_str(&responses[0])?;
    assert_eq!(workspace_response.id, 2);
    assert_eq!(workspace_response.result.len(), 3);

    assert!(
        workspace_response
            .result
            .iter()
            .any(|x| x.ends_with("testbed/file1.sql"))
    );
    assert!(
        workspace_response
            .result
            .iter()
            .any(|x| x.ends_with("testbed/file2.sql"))
    );
    assert!(
        workspace_response
            .result
            .iter()
            .any(|x| x.ends_with("testbed/nested/file3.sql"))
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn server_shuts_down_cleanly(
    #[future] mut stdio_server: TestServer,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdio_server = stdio_server.await;

    // Send shutdown request
    stdio_server
        .write_message(r#"{"jsonrpc":"2.0","id":99,"method":"shutdown"}"#)
        .await?;

    stdio_server.wait_for_messages(1).await;

    let exit = Request::build("exit").finish();
    // Send exit notification
    stdio_server
        .write_message(serde_json::to_string(&exit)?.as_str())
        .await?;

    // The server process should terminate within a reasonable time
    let result = tokio::time::timeout(Duration::from_secs(5), stdio_server.child.wait()).await;

    match result {
        Ok(Ok(status)) => {
            assert!(
                status.success(),
                "Server exited with non-zero status: {status}"
            );
        }
        Ok(Err(e)) => panic!("Failed to wait on server process: {e}"),
        Err(_) => panic!(
            "Server failed to shut down within 5 seconds - io_threads.join() is likely hanging"
        ),
    }

    Ok(())
}
