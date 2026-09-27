use crate::{
    server::LsServer,
    transport::{BaseLsHandle, LsHandle},
};
use assert_cmd::cargo::cargo_bin;
use rstest::{fixture, rstest};
use tower_lsp_server::{LanguageServer, ls_types::{InitializeParams, InitializedParams}};
use std::process::Stdio;
use tokio::process::Command;

#[fixture]
async fn bridge() -> LsServer<BaseLsHandle> {
    /*let command = CargoBuild::new()
        .current_release()
        .current_target()
        .bin("lsp-bridge")
        .run()
        .unwrap()
        .command();

    println!("{:?} {:?} {:?}", command, command.get_current_dir(), command.get_args());*/

    let curr_dir = std::env::current_dir().expect("Failed to fetch current directory");
    let bin = cargo_bin("lsp-bridge");
    println!("{bin:?}");
    let mut bridge = Command::new(bin)
        .current_dir(curr_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Unable to run lsp-bridge");

    let handle =
        BaseLsHandle::from_stdio(bridge.stdout.take().unwrap(), bridge.stdin.take().unwrap());
    LsServer::new(handle)
}

#[rstest]
#[tokio::test(flavor="current_thread")]
async fn bridge_shuts_down_gracefully(
    #[future] mut bridge: LsServer<BaseLsHandle>
) -> Result<(), Box<dyn std::error::Error>> {
    let bridge = bridge.await;

    bridge.initialize(InitializeParams {
        process_id: Some(std::process::id()),
        ..Default::default()
    }).await?;

    bridge.initialized(InitializedParams {}).await;

    bridge.shutdown().await?;

    bridge.exit().await;
    Ok(())
}