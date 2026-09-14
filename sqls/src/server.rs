use crate::error::Error;
use crate::lsp_service_ext::{ClientSocketExt, LspServiceExt};
use crate::util::convert::{convert_text_document_content_change_event, convert_ts_range};
use dashmap::mapref::one::Ref;
use dashmap::{DashMap, Entry};
use log::info;
use std::sync::Arc;
use texter::core::text::Text;
use texter::tree_sitter::StreamingIterator;
use texter::tree_sitter::{self, Parser, Query, QueryCursor, Tree};
use tokio::sync::RwLock;
use tower_lsp_server::jsonrpc::Result as JrpcResult;
use tower_lsp_server::ls_types::{
    Diagnostic, DiagnosticSeverity, DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams, NumberOrString, Position, PositionEncodingKind, Range, TextDocumentContentChangeEvent, TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions, Uri,
};
use tower_lsp_server::ls_types::{
    InitializeParams, InitializeResult, MessageType, ServerCapabilities, ServerInfo,
};
use tower_lsp_server::{Client, LanguageServer};

#[derive(Debug, Clone)]
pub struct Document {
    texter: Text,
    tree: Tree,
}

impl Document {
    fn split_mut(&mut self) -> (&mut Text, &mut Tree) {
        let text_ptr: *mut Text = &mut self.texter;
        let tree_ptr: *mut Tree = &mut self.tree;

        // SAFETY: These fields should never overlap in memory.
        unsafe { (&mut (*text_ptr), &mut (*tree_ptr)) }
    }

    pub async fn update(
        &mut self,
        change: TextDocumentContentChangeEvent,
        parser: Arc<RwLock<Parser>>
    ) -> Result<(), Error> {
        let (texter, tree) = self.split_mut();
        texter.update(convert_text_document_content_change_event(change), tree)?;
        *tree = parser.write().await.parse(texter.text.as_bytes(), Some(tree)).ok_or_else(|| Error::TreeSitterParserError)?;
        Ok(())
    }

    pub fn compute_diagnostics(&self) -> Vec<Diagnostic> {
        let error_query: Query = Query::new(&self.tree.language(), "[(ERROR) @error-node (MISSING) @missing-node]").unwrap(); // Error should always be a valid query
        let mut query_cursor = QueryCursor::new();
        let mut matches = query_cursor.matches(
            &error_query,
            self.tree.root_node(),
            self.texter.text.as_bytes(),
        );

        let mut result = vec![];
        while let Some(m) = matches.next() {
            let node = m.captures[0].node;
            let range = convert_ts_range(node.range());
            let message = if m.pattern_index == 0 { "Unexpected Token".to_string() } else { format!("Missing {}", node.child(0).unwrap().to_string()) };
            result.push(Diagnostic {
                range: range,
                severity: Some(DiagnosticSeverity::ERROR),
                source: Some("SQL Tree Sitter".to_owned()),
                message: message,
                ..Default::default()
            });
        }

        result
    }
}

pub struct File {
    pub url: Uri,
    pub document: Document,
}

impl File {
    pub fn new(url: Uri, document: Document) -> Self {
        File { url, document }
    }
}

pub struct Backend {
    client: Client,
    parser: Arc<RwLock<Parser>>,
    files: Arc<DashMap<Uri, File>>,
    encoding: fn(String) -> Text,
}

#[derive(Default)]
pub struct BackendBuilder {
    client: Option<Client>,
    files: DashMap<Uri, File>,
    encoding: Option<fn(String) -> Text>,
}

impl Backend {
    pub fn builder() -> BackendBuilder {
        BackendBuilder::default()
    }

    pub async fn log_runtime_error(&self, error: Error) {
        self.client
            .log_message(MessageType::ERROR, error.to_string())
            .await;
    }

    async fn add_file_from_texter(
        &self,
        url: &Uri,
        texter: Text,
    ) -> Result<Ref<'_, Uri, File>, Error> {
        let tree = self
            .parser
            .write()
            .await
            .parse(texter.text.as_bytes(), None)
            .ok_or_else(|| Error::TreeSitterParserError)?;

        let document = Document { texter, tree };
        let file = File::new(url.clone(), document);

        match self.files.entry(url.clone()) {
            Entry::Occupied(_) => Err(Error::FileAlreadyExists { uri: url.clone() })?,
            Entry::Vacant(entry) => {
                entry.insert(file);
            }
        };
        Ok(self.files.get(url).unwrap())
    }

    async fn update(
        &self,
        url: &Uri,
        changes: &[TextDocumentContentChangeEvent],
    ) -> Result<Option<Ref<'_, Uri, File>>, Error> {
        if let Some(mut file) = self.files.get_mut(url) {
            for change in changes {
                file.document.update(change.clone(), self.parser.clone()).await?;
            }
            info!(
                "Updated file: {}, new text: {}, new ast: {}", 
                file.key().as_str(), 
                serde_json::to_string(&file.document.texter.text).unwrap(), 
                file.document.tree.root_node().to_sexp()
            );
            Ok(Some(file.downgrade()))
        } else {
            Ok(None)
        }
    }

    pub async fn stdio() {
        let backend_builder = Backend::builder().encoding(Text::new);

        let (lsp_service, socket) =
            LspServiceExt::new(move |client| backend_builder.client(client).build().unwrap());
        let socket = ClientSocketExt::new(socket);
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();
        tower_lsp_server::Server::new(stdin, stdout, socket)
            .serve(lsp_service)
            .await;

        eprintln!("Shutting down server");
    }
}

impl BackendBuilder {
    pub fn client(mut self, client: Client) -> Self {
        self.client = Some(client);
        self
    }

    pub fn encoding(mut self, encoding: fn(String) -> Text) -> Self {
        self.encoding = Some(encoding);
        self
    }

    pub fn build(self) -> Option<Backend> {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_sequel::LANGUAGE.into())
            .ok()?;

        Some(Backend {
            client: self.client?,
            parser: Arc::new(RwLock::new(parser)),
            files: Arc::new(self.files),
            encoding: self.encoding?,
        })
    }
}

impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> JrpcResult<InitializeResult> {
        let position_encoding = match params.capabilities.general {
            Some(general) => match general.position_encodings {
                Some(encodings) => if encodings.contains(&PositionEncodingKind::UTF8) { PositionEncodingKind::UTF8 } else { PositionEncodingKind::UTF16 },
                None => PositionEncodingKind::UTF16
            },
            None => PositionEncodingKind::UTF16
        };
        return Ok(InitializeResult {
            capabilities: ServerCapabilities {
                position_encoding: Some(position_encoding),
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::INCREMENTAL),
                        ..Default::default()
                    }
                )),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "sqls".to_string(),
                version: Some("0.1.0".to_string()),
            }),
            offset_encoding: None,
        });
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        if params.text_document.language_id == "sql" {
            let texter = (self.encoding)(params.text_document.text);
            match self
                .add_file_from_texter(&params.text_document.uri, texter)
                .await
            {
                Ok(file) => {
                    let mut diagnostics = file.document.compute_diagnostics();
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position {
                                line: 0,
                                character: 0,
                            },
                            end: Position {
                                line: 0,
                                character: 1,
                            },
                        },
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: Some(NumberOrString::String("ParseError".to_string())),
                        code_description: None,
                        source: None,
                        message: "Hello World!".to_string(),
                        ..Default::default()
                    });
                    self.client
                        .publish_diagnostics(params.text_document.uri, diagnostics, None)
                        .await;
                }
                Err(error) => self.log_runtime_error(error).await,
            }
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        match self.update(&params.text_document.uri, &params.content_changes).await {
            Ok(Some(file)) => {
                let mut diagnostics = file.document.compute_diagnostics();
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position {
                            line: 0,
                            character: 0,
                        },
                        end: Position {
                            line: 0,
                            character: 1,
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::String("ParseError".to_string())),
                    code_description: None,
                    source: None,
                    message: "Hello World!".to_string(),
                    ..Default::default()
                });
                self.client
                    .publish_diagnostics(params.text_document.uri, diagnostics, None)
                    .await;
            }
            Err(error) => self.log_runtime_error(error.into()).await,
            Ok(None) => (),
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        if let Some((uri, _)) = self.files.remove(&params.text_document.uri) {
            self.client.publish_diagnostics(uri, vec![], None).await;
        }
    }

    async fn shutdown(&self) -> JrpcResult<()> {
        Ok(())
    }
}
