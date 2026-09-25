use serde::{Serialize, de::DeserializeOwned};
use std::{
    borrow::Cow, sync::{Arc, atomic::AtomicI64}, task::{Context, Poll},
};
use tokio::sync::{OwnedRwLockWriteGuard, RwLock};
use tower::Service;
use tower_lsp_server::{
    LanguageServer,
    jsonrpc::{self, Id, Request, Response, Result},
    ls_types::*,
};

#[derive(Debug, Clone, Default)]
struct UniqueIdIter {
    index: Arc<AtomicI64>,
    prefix: Option<&'static str>,
}

impl UniqueIdIter {
    pub fn with_prefix(prefix: &'static str) -> Self {
        Self {
            prefix: Some(prefix),
            ..Default::default()
        }
    }

    pub fn next(&self) -> Id {
        let current_index = self.index.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if let Some(prefix) = self.prefix {
            Id::String(format!("{prefix}{current_index}"))
        } else {
            Id::Number(current_index)
        }
    }
}

impl Iterator for UniqueIdIter {
    type Item = Id;

    fn next(&mut self) -> Option<Self::Item> {
        let current_index = self.index.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Some(if let Some(prefix) = self.prefix {
            Id::String(format!("{prefix}{current_index}"))
        } else {
            Id::Number(current_index)
        })
    }
}

#[derive(Debug, Clone)]
pub struct LsServer<S> {
    inner: Arc<RwLock<S>>,
    id_iter: UniqueIdIter,
}

impl<S> LsServer<S> {
    pub fn new(inner: S) -> Self {
        Self {
            inner: Arc::new(RwLock::new(inner)),
            id_iter: UniqueIdIter::with_prefix("bridge:"),
        }
    }

    pub async fn get_inner_mut(&self) -> OwnedRwLockWriteGuard<S> {
        self.inner.clone().write_owned().await
    } 

    pub fn poll_write(&self, cx: &mut Context<'_>) -> Poll<OwnedRwLockWriteGuard<S>> {
        match self.inner.clone().try_write_owned() {
            Ok(g) => Poll::Ready(g),
            Err(_) => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}

impl<S> LsServer<S>
where
    S: Service<Request, Response = Option<Response>>,
    S::Error: Into<jsonrpc::Error>,
{
    pub async fn send_request<M, P, R>(&self, method: M, params: P) -> Result<R>
    where
        M: Into<Cow<'static, str>>,
        P: Serialize,
        R: DeserializeOwned,
    {
        let request = Request::build(method)
            .params(serde_json::to_value(params).unwrap())
            .id(self.id_iter.next())
            .finish();
        let result = self
            .get_inner_mut()
            .await
            .call(request)
            .await
            .map_err(|e| e.into())?
            .unwrap();
        let (_, response) = result.into_parts();
        response.and_then(|r| serde_json::from_value(r).unwrap())
    }

    pub async fn send_notification<M, P>(&self, method: M, params: P)
    where
        M: Into<Cow<'static, str>>,
        P: Serialize,
    {
        let request = Request::build(method)
            .params(serde_json::to_value(params).unwrap())
            .finish();
        let _ = self.get_inner_mut().await.call(request).await;
    }

    pub async fn exit(&self) {
        self.send_notification("exit", ()).await
    }
}

impl<S> LanguageServer for LsServer<S>
where
    S: Service<Request, Response = Option<Response>> + Sync + Send + 'static,
    S::Error: Into<jsonrpc::Error>,
    S::Future: Send,
{ 
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        self.send_request("initialize", params).await
    }

    async fn initialized(&self, params: InitializedParams) {
        self.send_notification("initialized", params).await
    }

    async fn shutdown(&self) -> Result<()> {
        self.send_request("shutdown", ()).await
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.send_notification("textDocument/didOpen", params).await
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.send_notification("textDocument/didChange", params)
            .await
    }

    async fn will_save(&self, params: WillSaveTextDocumentParams) {
        self.send_notification("textDocument/willSave", params)
            .await
    }

    async fn will_save_wait_until(
        &self,
        params: WillSaveTextDocumentParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        self.send_request("textDocument/willSaveWaitUntil", params)
            .await
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.send_notification("textDocument/didSave", params).await
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.send_notification("textDocument/didClose", params)
            .await
    }

    async fn notebook_did_open(&self, params: DidOpenNotebookDocumentParams) {
        self.send_notification("notebookDocument/didOpen", params)
            .await
    }

    async fn notebook_did_change(&self, params: DidChangeNotebookDocumentParams) {
        self.send_notification("notebookDocument/didChange", params)
            .await
    }

    async fn notebook_did_save(&self, params: DidSaveNotebookDocumentParams) {
        self.send_notification("notebookDocument/didSave", params)
            .await
    }

    async fn notebook_did_close(&self, params: DidCloseNotebookDocumentParams) {
        self.send_notification("notebookDocument/didClose", params)
            .await
    }

    async fn goto_declaration(
        &self,
        params: request::GotoDeclarationParams,
    ) -> Result<Option<request::GotoDeclarationResponse>> {
        self.send_request("textDocument/declaration", params).await
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        self.send_request("textDocument/definition", params).await
    }

    async fn goto_type_definition(
        &self,
        params: request::GotoTypeDefinitionParams,
    ) -> Result<Option<request::GotoTypeDefinitionResponse>> {
        self.send_request("textDocument/typeDefinition", params)
            .await
    }

    async fn goto_implementation(
        &self,
        params: request::GotoImplementationParams,
    ) -> Result<Option<request::GotoImplementationResponse>> {
        self.send_request("textDocument/implementation", params)
            .await
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        self.send_request("textDocument/references", params).await
    }

    async fn prepare_call_hierarchy(
        &self,
        params: CallHierarchyPrepareParams,
    ) -> Result<Option<Vec<CallHierarchyItem>>> {
        self.send_request("textDocument/prepareCallHierarchy", params)
            .await
    }

    async fn incoming_calls(
        &self,
        params: CallHierarchyIncomingCallsParams,
    ) -> Result<Option<Vec<CallHierarchyIncomingCall>>> {
        self.send_request("callHierarchy/incomingCalls", params)
            .await
    }

    async fn outgoing_calls(
        &self,
        params: CallHierarchyOutgoingCallsParams,
    ) -> Result<Option<Vec<CallHierarchyOutgoingCall>>> {
        self.send_request("callHierarchy/outgoingCalls", params)
            .await
    }

    async fn prepare_type_hierarchy(
        &self,
        params: TypeHierarchyPrepareParams,
    ) -> Result<Option<Vec<TypeHierarchyItem>>> {
        self.send_request("textDocument/prepareTypeHierarchy", params)
            .await
    }

    async fn supertypes(
        &self,
        params: TypeHierarchySupertypesParams,
    ) -> Result<Option<Vec<TypeHierarchyItem>>> {
        self.send_request("typeHierarchy/supertypes", params).await
    }

    async fn subtypes(
        &self,
        params: TypeHierarchySubtypesParams,
    ) -> Result<Option<Vec<TypeHierarchyItem>>> {
        self.send_request("typeHierarchy/subtypes", params).await
    }

    async fn document_highlight(
        &self,
        params: DocumentHighlightParams,
    ) -> Result<Option<Vec<DocumentHighlight>>> {
        self.send_request("textDocument/documentHighlight", params)
            .await
    }

    async fn document_link(&self, params: DocumentLinkParams) -> Result<Option<Vec<DocumentLink>>> {
        self.send_request("textDocument/documentLink", params).await
    }

    async fn document_link_resolve(&self, params: DocumentLink) -> Result<DocumentLink> {
        self.send_request("documentLink/resolve", params).await
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        self.send_request("textDocument/hover", params).await
    }

    async fn code_lens(&self, params: CodeLensParams) -> Result<Option<Vec<CodeLens>>> {
        self.send_request("textDocument/codeLens", params).await
    }

    async fn code_lens_resolve(&self, params: CodeLens) -> Result<CodeLens> {
        self.send_request("codeLens/resolve", params).await
    }

    async fn folding_range(&self, params: FoldingRangeParams) -> Result<Option<Vec<FoldingRange>>> {
        self.send_request("textDocument/foldingRange", params).await
    }

    async fn selection_range(
        &self,
        params: SelectionRangeParams,
    ) -> Result<Option<Vec<SelectionRange>>> {
        self.send_request("textDocument/selectionRange", params)
            .await
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        self.send_request("textDocument/documentSymbol", params)
            .await
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        self.send_request("textDocument/semanticTokens/full", params)
            .await
    }

    async fn semantic_tokens_full_delta(
        &self,
        params: SemanticTokensDeltaParams,
    ) -> Result<Option<SemanticTokensFullDeltaResult>> {
        self.send_request("textDocument/semanticTokens/full/delta", params)
            .await
    }

    async fn semantic_tokens_range(
        &self,
        params: SemanticTokensRangeParams,
    ) -> Result<Option<SemanticTokensRangeResult>> {
        self.send_request("textDocument/semanticTokens/range", params)
            .await
    }

    async fn inline_value(&self, params: InlineValueParams) -> Result<Option<Vec<InlineValue>>> {
        self.send_request("textDocument/inlineValue", params).await
    }

    async fn inlay_hint(&self, params: InlayHintParams) -> Result<Option<Vec<InlayHint>>> {
        self.send_request("textDocument/inlayHint", params).await
    }

    async fn inlay_hint_resolve(&self, params: InlayHint) -> Result<InlayHint> {
        self.send_request("inlayHint/resolve", params).await
    }

    async fn moniker(&self, params: MonikerParams) -> Result<Option<Vec<Moniker>>> {
        self.send_request("textDocument/moniker", params).await
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        self.send_request("textDocument/completion", params).await
    }

    async fn completion_resolve(&self, params: CompletionItem) -> Result<CompletionItem> {
        self.send_request("completionItem/resolve", params).await
    }

    async fn diagnostic(
        &self,
        params: DocumentDiagnosticParams,
    ) -> Result<DocumentDiagnosticReportResult> {
        self.send_request("textDocument/diagnostic", params).await
    }

    async fn workspace_diagnostic(
        &self,
        params: WorkspaceDiagnosticParams,
    ) -> Result<WorkspaceDiagnosticReportResult> {
        self.send_request("workspace/diagnostic", params).await
    }

    async fn signature_help(&self, params: SignatureHelpParams) -> Result<Option<SignatureHelp>> {
        self.send_request("textDocument/signatureHelp", params)
            .await
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        self.send_request("textDocument/codeAction", params).await
    }

    async fn code_action_resolve(&self, params: CodeAction) -> Result<CodeAction> {
        self.send_request("codeAction/resolve", params).await
    }

    async fn document_color(&self, params: DocumentColorParams) -> Result<Vec<ColorInformation>> {
        self.send_request("textDocument/documentColor", params)
            .await
    }

    async fn color_presentation(
        &self,
        params: ColorPresentationParams,
    ) -> Result<Vec<ColorPresentation>> {
        self.send_request("textDocument/colorPresentation", params)
            .await
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        self.send_request("textDocument/formatting", params).await
    }

    async fn range_formatting(
        &self,
        params: DocumentRangeFormattingParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        self.send_request("textDocument/rangeFormatting", params)
            .await
    }

    async fn on_type_formatting(
        &self,
        params: DocumentOnTypeFormattingParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        self.send_request("textDocument/onTypeFormatting", params)
            .await
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        self.send_request("textDocument/rename", params).await
    }

    async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Option<PrepareRenameResponse>> {
        self.send_request("textDocument/prepareRename", params)
            .await
    }

    async fn linked_editing_range(
        &self,
        params: LinkedEditingRangeParams,
    ) -> Result<Option<LinkedEditingRanges>> {
        self.send_request("textDocument/linkedEditingRange", params)
            .await
    }

    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<WorkspaceSymbolResponse>> {
        self.send_request("workspace/symbol", params).await
    }

    async fn symbol_resolve(&self, params: WorkspaceSymbol) -> Result<WorkspaceSymbol> {
        self.send_request("workspaceSymbol/resolve", params).await
    }

    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        self.send_notification("workspace/didChangeConfiguration", params)
            .await
    }

    async fn did_change_workspace_folders(&self, params: DidChangeWorkspaceFoldersParams) {
        self.send_notification("workspace/didChangeWorkspaceFolders", params)
            .await
    }

    async fn will_create_files(&self, params: CreateFilesParams) -> Result<Option<WorkspaceEdit>> {
        self.send_request("workspace/willCreateFiles", params).await
    }

    async fn did_create_files(&self, params: CreateFilesParams) {
        self.send_notification("workspace/didCreateFiles", params)
            .await
    }

    async fn will_rename_files(&self, params: RenameFilesParams) -> Result<Option<WorkspaceEdit>> {
        self.send_request("workspace/willRenameFiles", params).await
    }

    async fn did_rename_files(&self, params: RenameFilesParams) {
        self.send_notification("workspace/didRenameFiles", params)
            .await
    }

    async fn will_delete_files(&self, params: DeleteFilesParams) -> Result<Option<WorkspaceEdit>> {
        self.send_request("workspace/willDeleteFiles", params).await
    }

    async fn did_delete_files(&self, params: DeleteFilesParams) {
        self.send_notification("workspace/didDeleteFiles", params)
            .await
    }

    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        self.send_notification("workspace/didChangeWatchedFiles", params)
            .await
    }

    async fn execute_command(&self, params: ExecuteCommandParams) -> Result<Option<LSPAny>> {
        self.send_request("workspace/executeCommand", params).await
    }
}
