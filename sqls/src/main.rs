#![recursion_limit = "256"]

use auto_lsp::configure_parsers;
use auto_lsp::default::db::BaseDatabase;
use auto_lsp::default::server::file_events::{changed_watched_files, open_text_document};
use auto_lsp::lsp_server::RequestId;
use auto_lsp::lsp_types::NumberOrString;
use auto_lsp::lsp_types::notification::{
    Cancel, DidChangeWatchedFiles, DidCloseTextDocument, DidOpenTextDocument, DidSaveTextDocument,
    LogTrace, SetTrace,
};
use auto_lsp::server::notification_registry::NotificationRegistry;
use auto_lsp::texter::core::text::Text;
use fastrace::collector::{Config, ConsoleReporter};
use sqls::generated::Program;
use sqls::server::Backend;
use std::error::Error;
use std::panic::RefUnwindSafe;
use tower_lsp_server::ls_types::OneOf::Left;
use tower_lsp_server::ls_types::{WorkspaceFoldersServerCapabilities, WorkspaceServerCapabilities};
use tree_sitter_sequel::LANGUAGE;

configure_parsers!(
    SQL_PARSERS,
    "sql" => {
        language: LANGUAGE,
        ast_root: Program
    }
);

pub trait ExtendDb: BaseDatabase {
    fn get_urls(&self) -> Vec<String> {
        self.get_files()
            .iter()
            .map(|file| file.url(self).to_string())
            .collect()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    stderrlog::new()
        .modules([module_path!(), "sqls"])
        .verbosity(4)
        .init()
        .unwrap();

    fastrace::set_reporter(ConsoleReporter, Config::default());

    let backend_builder = Backend::builder()
        .parsers(&SQL_PARSERS)
        .capabilities(tower_lsp_server::ls_types::ServerCapabilities {
            workspace: Some(WorkspaceServerCapabilities {
                workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                    change_notifications: Some(Left(true)),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        })
        .info(tower_lsp_server::ls_types::ServerInfo {
            name: "sqls".to_string(),
            version: Some("0.1.0".to_string()),
        })
        .encoding(Text::new)
        .extensions(
            [("sql".to_string(), "sql".to_string())]
                .into_iter()
                .collect(),
        );

    let (lsp_service, socket) = tower_lsp_server::LspService::new(move |client| {
        backend_builder.client(client).build().unwrap()
    });
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    tower_lsp_server::Server::new(stdin, stdout, socket)
        .serve(lsp_service)
        .await;

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
