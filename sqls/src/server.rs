use std::collections::HashMap;
use std::io::Read;
use std::sync::Arc;

use auto_lsp::core::document::Document;
use auto_lsp::core::errors::{
    DataBaseError, ExtensionError, FileSystemError, RuntimeError, TreeSitterError,
};
use auto_lsp::core::parsers::Parsers;
use auto_lsp::default::db::{BaseDatabase, BaseDb};
use auto_lsp::default::server::workspace_init::WorkspaceInit;
use auto_lsp::lsp_types::Url;
use auto_lsp::texter::core::text::Text;
use dashmap::{DashMap, Entry};
use rayon::prelude::*;
use tower_lsp_server::jsonrpc::Result as JrpcResult;
use tower_lsp_server::ls_types::{InitializeParams, InitializeResult, MessageType, ServerCapabilities, ServerInfo};
use tower_lsp_server::{Client, LanguageServer};
use walkdir::WalkDir;

/// Get the extension of a file from a [`Url`] path
#[cfg(windows)]
pub(crate) fn get_extension(path: &Url) -> Result<String, FileSystemError> {
    // Ensure the host is either empty or "localhost" on Windows
    if let Some(host) = path.host_str() {
        if !host.is_empty() && host != "localhost" {
            return Err(FileSystemError::FileUrlHost {
                host: host.to_string(),
                path: path.clone(),
            });
        }
    }

    path.to_file_path()
        .map_err(|_| FileSystemError::FileUrlToFilePath { path: path.clone() })?
        .extension()
        .map_or_else(
            || Err(FileSystemError::FileExtension { path: path.clone() }),
            |ext| Ok(ext.to_string_lossy().to_string()),
        )
}

#[cfg(not(windows))]
pub(crate) fn get_extension(path: &Url) -> Result<String, FileSystemError> {
    path.to_file_path()
        .map_err(|_| FileSystemError::FileUrlToFilePath { path: path.clone() })?
        .extension()
        .map_or_else(
            || Err(FileSystemError::FileExtension { path: path.clone() }),
            |ext| Ok(ext.to_string_lossy().to_string()),
        )
}

pub trait ExtendDb: BaseDatabase {
    fn get_urls(&self) -> Vec<String> {
        self.get_files()
            .iter()
            .map(|file| file.url(self).to_string())
            .collect()
    }
}

impl ExtendDb for BaseDb {}

pub struct File {
    pub url: Url,
    pub parsers: &'static Parsers,
    pub document: Arc<Document>,
}

impl File {
    pub fn new(url: Url, parsers: &'static Parsers, document: Arc<Document>) -> Self {
        File {
            url,
            parsers,
            document,
        }
    }
}

pub struct Backend {
    client: Client,
    parsers: &'static HashMap<&'static str, Parsers>,
    capabilities: ServerCapabilities,
    info: ServerInfo,
    files: Arc<DashMap<Url, File>>,
    encoding: fn(String) -> Text,
    extensions: HashMap<String, String>,
}

pub struct BackendMut<'a> {
    inner: &'a Backend,
}

#[derive(Default)]
pub struct BackendBuilder {
    client: Option<Client>,
    parsers: Option<&'static HashMap<&'static str, Parsers>>,
    capabilities: Option<ServerCapabilities>,
    info: Option<ServerInfo>,
    files: DashMap<Url, File>,
    encoding: Option<fn(String) -> Text>,
    extensions: HashMap<String, String>,
}

impl Backend {
    pub fn builder() -> BackendBuilder {
        BackendBuilder::default()
    }

    pub async fn log_runtime_error(&self, error: RuntimeError) {
        self.client
            .log_message(MessageType::ERROR, error.to_string())
            .await;
    }

    pub fn write<'a>(&'a self) -> BackendMut<'a> {
        BackendMut { inner: self }
    }

    pub fn get_files(&self) -> Arc<DashMap<Url, File>> {
        self.files.clone()
    }

    fn add_file_from_texter(
        &self,
        parsers: &'static Parsers,
        url: &Url,
        texter: Text,
    ) -> Result<(), DataBaseError> {
        let tree = parsers
            .parser
            .write()
            .parse(texter.text.as_bytes(), None)
            .ok_or_else(|| DataBaseError::from((url, TreeSitterError::TreeSitterParser)))?;

        let document = Document { texter, tree };
        let file = File::new(url.clone(), parsers, Arc::new(document));

        match self.get_files().entry(url.clone()) {
            Entry::Occupied(_) => Err(DataBaseError::FileAlreadyExists { uri: url.clone() }),
            Entry::Vacant(entry) => {
                entry.insert(file);
                Ok(())
            }
        }
    }
}

impl BackendBuilder {
    pub fn client(mut self, client: Client) -> Self {
        self.client = Some(client);
        self
    }

    pub fn parsers(mut self, parsers: &'static HashMap<&'static str, Parsers>) -> Self {
        self.parsers = Some(parsers);
        self
    }

    pub fn capabilities(mut self, capabilities: ServerCapabilities) -> Self {
        self.capabilities = Some(capabilities);
        self
    }

    pub fn info(mut self, info: ServerInfo) -> Self {
        self.info = Some(info);
        self
    }

    pub fn encoding(mut self, encoding: fn(String) -> Text) -> Self {
        self.encoding = Some(encoding);
        self
    }

    pub fn extensions(mut self, extensions: HashMap<String, String>) -> Self {
        self.extensions = extensions;
        self
    }

    pub fn build(self) -> Option<Backend> {
        Some(Backend {
            client: self.client?,
            parsers: self.parsers?,
            capabilities: self.capabilities?,
            info: self.info?,
            files: Arc::new(self.files),
            encoding: self.encoding?,
            extensions: self.extensions,
        })
    }
}

impl<'a> WorkspaceInit for BackendMut<'a> {
    fn init_workspace(
        &mut self,
        params: auto_lsp::lsp_types::InitializeParams,
    ) -> Result<
        Vec<Result<(), auto_lsp::core::errors::RuntimeError>>,
        auto_lsp::core::errors::RuntimeError,
    > {
        let mut errors: Vec<Result<(), RuntimeError>> = vec![];

        if let Some(folders) = params.workspace_folders {
            let files = folders
                .into_iter()
                .flat_map(|folder| {
                    WalkDir::new(folder.uri.path())
                        .into_iter()
                        .filter_map(Result::ok)
                        .filter(|entry| {
                            entry.file_type().is_file()
                                && entry.path().extension().is_some_and(|ext| {
                                    self.inner
                                        .extensions
                                        .contains_key(ext.to_string_lossy().as_ref())
                                })
                        })
                })
                .collect::<Vec<_>>();

            errors.extend(rayon_par_bridge::par_bridge(
                16,
                files.into_par_iter(),
                |file_iter| {
                    file_iter
                        .map(|file| match self.read_file(&file.into_path()) {
                            Ok((parsers, url, text)) => self
                                .inner
                                .add_file_from_texter(parsers, &url, text)
                                .map_err(RuntimeError::from),
                            Err(err) => Err(RuntimeError::from(err)),
                        })
                        .collect::<Vec<_>>()
                },
            ));
        }

        Ok(errors)
    }

    fn read_file(
        &self,
        file: &std::path::Path,
    ) -> Result<
        (
            &'static auto_lsp::core::parsers::Parsers,
            auto_lsp::lsp_types::Url,
            Text,
        ),
        auto_lsp::core::errors::FileSystemError,
    > {
        let url = Url::from_file_path(file).map_err(|_| FileSystemError::FilePathToUrl {
            path: file.to_path_buf(),
        })?;

        let mut open_file = std::fs::File::open(file).map_err(|e| FileSystemError::FileOpen {
            path: url.clone(),
            error: e.to_string(),
        })?;
        let mut buffer = String::new();
        open_file
            .read_to_string(&mut buffer)
            .map_err(|e| FileSystemError::FileRead {
                path: url.clone(),
                error: e.to_string(),
            })?;

        let extension = get_extension(&url)?;

        let text = (self.inner.encoding)(buffer.to_string());
        let extension = match self.inner.extensions.get(&extension) {
            Some(extension) => extension,
            None => {
                return Err(FileSystemError::from(ExtensionError::UnknownExtension {
                    extension: extension.clone(),
                    available: self.inner.extensions.clone(),
                }));
            }
        };

        let parsers = self
            .inner
            .parsers
            .get(extension.as_str())
            .ok_or_else(|| {
                FileSystemError::from(ExtensionError::UnknownParser {
                    extension: extension.clone(),
                    available: self.inner.parsers.keys().cloned().collect(),
                })
            })?;
        Ok((parsers, url, text))
    }
}

impl LanguageServer for Backend {
    async fn initialize(&self, _params: InitializeParams) -> JrpcResult<InitializeResult> {
        /*let errors = self.write().init_workspace(params).map_or_else(
            |err| vec![err],
            |xs| xs.into_iter().flat_map(|x| x.err()).collect(),
        );
        for error in errors {
            self.log_runtime_error(error).await;
        }*/
        return Ok(InitializeResult {
            capabilities: self.capabilities.clone(),
            server_info: Some(self.info.clone()),
            offset_encoding: None
        });
    }

    async fn shutdown(&self) -> JrpcResult<()> {
        Ok(())
    }
}
