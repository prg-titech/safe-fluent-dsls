use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::Mutex as SyncMutex;

use crate::iter::TreeIterator;

use lazy_static::lazy_static;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result as RpcResult;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
use tree_sitter::Range as TSRange;
use tree_sitter::{InputEdit, Parser, Point, Tree};

lazy_static! {
    static ref PARSER: SyncMutex<Parser> = {
        let mut parser = tree_sitter::Parser::new();
        let language = tree_sitter_sequel::LANGUAGE;
        parser
            .set_language(&language.into())
            .expect("Unable to load SQL grammar");
        SyncMutex::new(parser)
    };
}

struct VersionOutOfSyncError;

#[derive(Debug)]
struct Document {
    uri: Url,
    version: i32,
    text: String,
    line_breaks: Vec<usize>,
    ast: Tree,
}

impl Document {
    fn position_to_point(position: Position) -> Point {
        Point {
            row: position.line as usize,
            column: position.character as usize,
        }
    }

    fn point_to_position(point: Point) -> Position {
        Position {
            line: point.row as u32,
            character: point.column as u32,
        }
    }

    fn point_to_byte_index(&self, point: Point) -> usize {
        self.line_breaks[point.row] + point.column
    }

    fn find_line_breaks(text: &str) -> Vec<usize> {
        text.split('\n').fold(vec![0], |mut acc, x| {
            acc.push(acc.last().unwrap() + x.len());
            acc
        })
    }

    fn compute_delta(text: &str) -> Point {
        let line_breaks = Document::find_line_breaks(text);
        Point {
            column: line_breaks[line_breaks.len() - 1] - line_breaks[line_breaks.len() - 2],
            row: line_breaks.len() - 2,
        }
    }

    fn point_plus_delta(point: Point, delta: Point) -> Point {
        if delta.row == 0 {
            Point {
                column: point.column + delta.column,
                row: point.row,
            }
        } else {
            Point {
                column: delta.column,
                row: point.row + delta.row,
            }
        }
    }

    fn tree_range_to_lsp_range(tree_range: TSRange) -> Range {
        Range {
            start: Document::point_to_position(tree_range.start_point),
            end: Document::point_to_position(tree_range.end_point),
        }
    }

    fn compute_input_edit(&self, old_range: &Range, new_text: &str) -> InputEdit {
        let start_position = Document::position_to_point(old_range.start);
        let old_end_position = Document::position_to_point(old_range.end);
        let new_end_position =
            Document::point_plus_delta(start_position, Document::compute_delta(new_text));
        let start_byte = self.point_to_byte_index(start_position);
        InputEdit {
            start_byte,
            old_end_byte: self.point_to_byte_index(old_end_position),
            new_end_byte: start_byte + new_text.len(),
            start_position,
            old_end_position,
            new_end_position,
        }
    }

    fn apply_change(
        &mut self,
        new_version: i32,
        change_event: &TextDocumentContentChangeEvent,
    ) -> Result<(), VersionOutOfSyncError> {
        if new_version < self.version {
            return Err(VersionOutOfSyncError);
        }
        if let Some(range) = change_event.range {
            let input_edit = self.compute_input_edit(&range, &change_event.text);
            self.text.replace_range(
                input_edit.start_byte..input_edit.old_end_byte,
                &change_event.text,
            );
            self.ast.edit(&input_edit);
            self.ast = PARSER
                .lock()
                .unwrap()
                .parse(self.text.as_bytes(), Some(&self.ast))
                .unwrap();
        } else {
            panic!();
        }
        Ok(())
    }

    fn apply_changes(
        &mut self,
        changes: &DidChangeTextDocumentParams,
    ) -> Result<(), VersionOutOfSyncError> {
        for change in changes.content_changes.iter() {
            self.apply_change(changes.text_document.version, change)?;
        }
        self.version = changes.text_document.version;
        Ok(())
    }

    fn compute_diagnostics(&self) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];
        if self.ast.root_node().has_error() {
            for node in TreeIterator::new(self.ast.walk()) {
                if node.is_error() {
                    let diagnostic = Diagnostic::new_simple(
                        Document::tree_range_to_lsp_range(node.range()),
                        "".to_owned(),
                    );
                    diagnostics.push(diagnostic);
                }
            }
        }
        diagnostics
    }
}

impl From<TextDocumentItem> for Document {
    fn from(value: TextDocumentItem) -> Self {
        let line_breaks = Document::find_line_breaks(&value.text);
        let ast: Tree = PARSER
            .lock()
            .unwrap()
            .parse(value.text.as_bytes(), None)
            .unwrap();
        Document {
            uri: value.uri,
            version: value.version,
            text: value.text,
            line_breaks: line_breaks,
            ast: ast,
        }
    }
}

#[derive(Debug)]
pub struct Sqls {
    client: Client,
    client_capabilities: Option<ClientCapabilities>,
    documents: HashMap<Url, Document>,
}

impl Sqls {
    pub fn new(client: Client) -> SqlsRef {
        let inner = Self {
            client,
            client_capabilities: None,
            documents: HashMap::new(),
        };
        SqlsRef {
            inner: Arc::new(Mutex::new(inner)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SqlsRef {
    inner: Arc<Mutex<Sqls>>,
}

#[tower_lsp::async_trait]
impl Deref for SqlsRef {
    type Target = Mutex<Sqls>;

    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for SqlsRef {
    async fn initialize(&self, params: InitializeParams) -> RpcResult<InitializeResult> {
        self.lock().await.client_capabilities = Some(params.capabilities);
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "sqls-rust".to_owned(),
                version: Some("0.1.0".to_owned()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {}

    async fn shutdown(&self) -> RpcResult<()> {
        self.lock()
            .await
            .client
            .log_message(MessageType::LOG, "Shutting down SQLS")
            .await;
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let document = params.text_document;
        let uri = document.uri.clone();
        if document.language_id != Sqls::language_id() {
            return;
        }

        let real_document: Document = document.into();
        let mut lock = self.lock().await;
        lock.client
            .log_message(
                MessageType::INFO,
                format!("Opened new document: {real_document:#?}"),
            )
            .await;
        lock.client
            .log_message(
                MessageType::INFO,
                format!("AST: {}", real_document.ast.root_node()),
            )
            .await;
        lock.documents
            .insert(real_document.uri.clone(), real_document);
        if lock.does_client_support_push_diagnostics() {
            lock.publish_diagnostics(uri).await;
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let mut lock = self.lock().await;
        match lock.documents.get_mut(&params.text_document.uri) {
            Some(d) => match d.apply_changes(&params) {
                Err(_) => {
                    lock.client
                        .log_message(
                            MessageType::ERROR,
                            format!("Document {} out of sync", params.text_document.uri),
                        )
                        .await
                }
                _ => (),
            },
            None => {
                lock.client
                    .log_message(
                        MessageType::ERROR,
                        format!("Document at uri {} not opened", params.text_document.uri),
                    )
                    .await
            }
        }
        lock.client
            .log_message(
                MessageType::INFO,
                format!(
                    "AST: {}",
                    lock.documents[&params.text_document.uri].ast.root_node()
                ),
            )
            .await;
        lock.publish_diagnostics(params.text_document.uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.lock()
            .await
            .documents
            .remove(&params.text_document.uri);
    }
}

impl Sqls {
    const fn language_id() -> &'static str {
        "sql"
    }

    fn does_client_support_push_diagnostics(&self) -> bool {
        if let Some(capabilities) = &self.client_capabilities {
            if let Some(text_document) = &capabilities.text_document {
                if let Some(_) = &text_document.publish_diagnostics {
                    return true;
                }
            }
        }
        false
    }

    async fn publish_diagnostics(&self, uri: Url) {
        let diagnostics = self.documents[&uri].compute_diagnostics();
        let version = Some(self.documents[&uri].version);
        self.client
            .publish_diagnostics(uri, diagnostics, version)
            .await;
    }
}
