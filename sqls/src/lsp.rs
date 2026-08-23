use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result as RpcResult;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
use tree_sitter::Tree;

#[derive(Debug)]
struct Document {
    uri: Url,
    version: i32,
    text: String,
    ast: Tree,
}

impl From<TextDocumentItem> for Document {
    fn from(value: TextDocumentItem) -> Self {
        let mut parser = tree_sitter::Parser::new();
        let language = tree_sitter_sequel::LANGUAGE;
        parser
            .set_language(&language.into())
            .expect("Unable to load SQL grammar");

        let ast: Tree = parser.parse(value.text.as_bytes(), None).unwrap();
        Document {
            uri: value.uri,
            version: value.version,
            text: value.text,
            ast: ast,
        }
    }
}

#[derive(Debug)]
pub struct Sqls {
    client: Client,
    documents: HashMap<Url, Document>,
}

impl Sqls {
    pub fn new(client: Client) -> SqlsRef {
        let inner = Self {
            client,
            documents: HashMap::new(),
        };
        SqlsRef {
            inner: Arc::new(Mutex::new(inner))
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
    async fn initialize(&self, _: InitializeParams) -> RpcResult<InitializeResult> {
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
        self.lock().await.client.log_message(MessageType::LOG, "Shutting down SQLS").await;
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let document = params.text_document;
        if document.language_id != Sqls::language_id() {
            return;
        }

        let real_document: Document = document.into();
        let mut lock = self.lock().await;
        lock.client
            .log_message(MessageType::INFO, format!("Opened new document: {real_document:#?}"))
            .await;
        lock.client
            .log_message(MessageType::INFO, format!("AST: {}", real_document.ast.root_node()))
            .await;
        lock.documents
            .insert(real_document.uri.clone(), real_document);
    }
}

impl Sqls {
    const fn language_id() -> &'static str {
        "sql"
    }
}
