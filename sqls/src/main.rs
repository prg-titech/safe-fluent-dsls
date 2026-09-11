#![recursion_limit = "256"]

mod generated;

use crate::generated::Program;
use auto_lsp::default::db::{BaseDatabase, BaseDb};
use auto_lsp::default::server::capabilities::WORKSPACE_PROVIDER;
use auto_lsp::default::server::file_events::{changed_watched_files, open_text_document};
use auto_lsp::default::server::workspace_init::WorkspaceInit;
use auto_lsp::lsp_server::{Connection, RequestId};
use auto_lsp::lsp_types::notification::{
    Cancel, DidChangeWatchedFiles, DidCloseTextDocument, DidOpenTextDocument, DidSaveTextDocument,
    LogTrace, SetTrace,
};
use auto_lsp::lsp_types::{NumberOrString, ServerCapabilities, ServerInfo};
use auto_lsp::server::Session;
use auto_lsp::server::notification_registry::NotificationRegistry;
use auto_lsp::server::request_registry::RequestRegistry;
use auto_lsp::{configure_parsers, server::options::InitOptions};
use fastrace::collector::{Config, ConsoleReporter};
use std::error::Error;
use std::panic::RefUnwindSafe;
use tree_sitter_sequel::LANGUAGE;

configure_parsers!(
    SQL_PARSERS,
    "sql" => {
        language: LANGUAGE,
        ast_root: Program
    }
);

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    stderrlog::new()
        .modules([module_path!(), "auto_lsp"])
        .verbosity(4)
        .init()
        .unwrap();

    fastrace::set_reporter(ConsoleReporter, Config::default());

    // Server options
    let options = InitOptions {
        parsers: &SQL_PARSERS,
        capabilities: ServerCapabilities {
            workspace: WORKSPACE_PROVIDER.clone(),
            ..Default::default()
        },
        server_info: Some(ServerInfo {
            name: "sqls".to_string(),
            version: Some("0.1.0".to_string()),
        }),
    };

    // Create the connection
    let (connection, io_threads) = Connection::stdio();
    // Create a database, either BaseDb or your own
    let db = BaseDb::default();

    // Create the session
    let (mut session, params) = Session::create(options, connection, db)?;

    // This is where you register your requests and notifications
    // See the handlers section for more information
    let mut request_registry = RequestRegistry::<BaseDb>::default();
    let mut notification_registry = NotificationRegistry::<BaseDb>::default();
    on_notifications(&mut notification_registry);

    // This will add all files available in the workspace.
    // The init_workspace is only available for databases that implement BaseDatabase or BaseDb
    let init_results = session.init_workspace(params)?;
    if !init_results.is_empty() {
        init_results.into_iter().for_each(|result| {
            if let Err(err) = result {
                eprintln!("{}", err);
            }
        });
    };

    // Run the server and wait for the two threads to end (typically by trigger LSP Exit event).
    session.main_loop(&mut request_registry, &mut notification_registry)?;
    io_threads.join()?;

    // Shut down gracefully.
    eprintln!("Shutting down server");
    Ok(())
}

fn on_notifications<Db: BaseDatabase + Clone + RefUnwindSafe>(
    registry: &mut NotificationRegistry<Db>,
) -> &mut NotificationRegistry<Db> {
    registry
        .on_mut::<DidOpenTextDocument, _>(|s, p| match p.text_document.language_id.as_str() {
            "python" => Ok(open_text_document(s, p)?),
            _ => Ok(()),
        })
        //.on_mut::<DidChangeTextDocument, _>(|s, p| Ok(change_text_document(s, p)?))
        .on_mut::<DidChangeWatchedFiles, _>(|s, p| Ok(changed_watched_files(s, p)?))
        .on_mut::<Cancel, _>(|s, p| {
            let id: RequestId = match p.id {
                NumberOrString::Number(id) => id.into(),
                NumberOrString::String(id) => id.into(),
            };
            if let Some(response) = s.req_queue.incoming.cancel(id) {
                s.connection.sender.send(response.into())?;
            }
            Ok(())
        })
        .on::<DidSaveTextDocument, _>(|_s, _p| Ok(()))
        .on::<DidCloseTextDocument, _>(|_s, _p| Ok(()))
        .on::<SetTrace, _>(|_s, _p| Ok(()))
        .on::<LogTrace, _>(|_s, _p| Ok(()))
}
