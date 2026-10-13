use futures::TryFutureExt;
use log::error;
use serde::Serialize;
use serde_json::json;
use std::{fmt::Display, task::{Context, Poll}};
use tower::{Layer, Service, ServiceExt};
use tower_lsp_server::{
    LanguageServer,
    jsonrpc::{self, Request, Response, Result},
    ls_types::*,
};

use crate::layer::{
    UniqueIdIter, client::{LanguageClient, LanguageHandle}, id_map::{MapIdLayer, MapIdService, map_id},
};

#[derive(Clone)]
pub struct LsLayer {
    map_id_layer: MapIdLayer,
}

impl LsLayer {
    pub fn new(prefix: &'static str) -> Self {
        Self {
            map_id_layer: map_id(prefix),
        }
    }
}

impl<S> Layer<S> for LsLayer {
    type Service = LsServer<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LsServer::new(inner, self.map_id_layer.prefix())
    }
}

#[derive(Debug)]
pub struct LsServer<S> {
    inner: MapIdService<S>,
    id_iter: UniqueIdIter,
}

impl<S: Clone> Clone for LsServer<S> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            id_iter: self.id_iter.clone(),
        }
    }
}

impl<S> LsServer<S> {
    pub fn new(inner: S, prefix: &'static str) -> Self {
        Self {
            inner: map_id(prefix).layer(inner),
            id_iter: UniqueIdIter::new(),
        }
    }
}

impl<S> LanguageHandle for LsServer<S>
where
    S: Service<Request, Response = Option<Response>> + Clone + Sync + Send + 'static,
    S::Error: Into<jsonrpc::Error>,
    S::Future: Send + 'static,
{
    async fn send_request<R>(&self, params: R::Params) -> jsonrpc::Result<R::Result>
    where
        R: request::Request,
    {
        let params = serde_json::to_value(params).map_err(|e| jsonrpc::Error {
            code: jsonrpc::ErrorCode::ParseError,
            message: e.to_string().into(),
            data: None,
        })?;

        let request = Request::build(R::METHOD)
            .id(self.id_iter.clone().next().unwrap())
            .params(params)
            .finish();

        let mut inner = self.inner.clone();
        let response = inner
            .ready()
            .await
            .map_err(|e| e.into())?
            .call(request)
            .await
            .map(|r| r.unwrap())
            .map_err(|e| e.into())?;

        let (_, result) = response.into_parts();
        result.and_then(|v| match serde_json::from_value(v) {
            Ok(r) => Ok(r),
            Err(e) => {
                if e.to_string().contains("expected unit") {
                    Ok(serde_json::from_value(json!(null)).unwrap())
                } else {
                    Err(jsonrpc::Error {
                        code: jsonrpc::ErrorCode::ParseError,
                        message: e.to_string().into(),
                        data: None,
                    })
                }
            }
        })
    }

    async fn send_notification<N>(&self, params: N::Params)
    where
        N: notification::Notification,
    {
        let params = match serde_json::to_value(params) {
            Ok(v) => v,
            Err(e) => {
                error!("Unable to serialize parameters with error: {e}");
                return;
            }
        };

        let request = Request::build(N::METHOD).params(params).finish();
        if self
            .clone()
            .ready()
            .and_then(|s| s.call(request))
            .await
            .is_err()
        {
            error!("failed to send notification");
        }
    }
}

impl<S> LsServer<S>
where
    S: Service<Request, Response = Option<Response>> + Clone + Sync + Send + 'static,
    S::Error: Into<jsonrpc::Error>,
    S::Future: Send + 'static,
{
    #[allow(unused)]
    pub async fn exit(&self) {
        self.send_notification::<notification::Exit>(()).await
    }
}

impl<S> Service<Request> for LsServer<S>
where
    S: Service<Request, Response = Option<Response>>,
    S::Future: Send + 'static,
{
    type Response = Option<Response>;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<core::result::Result<(), Self::Error>> {
        self.inner.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        self.inner.inner.call(req)
    }
}

impl<S> LanguageServer for LsServer<S>
where
    S: Service<Request, Response = Option<Response>> + Clone + Send + Sync + 'static,
    S::Error: Into<jsonrpc::Error>,
    S::Future: Send + 'static,
{
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        self.send_request::<request::Initialize>(params).await
    }

    async fn initialized(&self, params: InitializedParams) {
        self.send_notification::<notification::Initialized>(params)
            .await
    }

    async fn shutdown(&self) -> Result<()> {
        self.send_request::<request::Shutdown>(()).await
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.send_notification::<notification::DidOpenTextDocument>(params)
            .await
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.send_notification::<notification::DidChangeTextDocument>(params)
            .await
    }

    async fn will_save(&self, params: WillSaveTextDocumentParams) {
        self.send_notification::<notification::WillSaveTextDocument>(params)
            .await
    }

    async fn will_save_wait_until(
        &self,
        params: WillSaveTextDocumentParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        self.send_request::<request::WillSaveWaitUntil>(params)
            .await
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.send_notification::<notification::DidSaveTextDocument>(params)
            .await
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.send_notification::<notification::DidCloseTextDocument>(params)
            .await
    }

    async fn notebook_did_open(&self, params: DidOpenNotebookDocumentParams) {
        self.send_notification::<notification::DidOpenNotebookDocument>(params)
            .await
    }

    async fn notebook_did_change(&self, params: DidChangeNotebookDocumentParams) {
        self.send_notification::<notification::DidChangeNotebookDocument>(params)
            .await
    }

    async fn notebook_did_save(&self, params: DidSaveNotebookDocumentParams) {
        self.send_notification::<notification::DidSaveNotebookDocument>(params)
            .await
    }

    async fn notebook_did_close(&self, params: DidCloseNotebookDocumentParams) {
        self.send_notification::<notification::DidCloseNotebookDocument>(params)
            .await
    }

    async fn goto_declaration(
        &self,
        params: request::GotoDeclarationParams,
    ) -> Result<Option<request::GotoDeclarationResponse>> {
        self.send_request::<request::GotoDeclaration>(params).await
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        self.send_request::<request::GotoDefinition>(params).await
    }

    async fn goto_type_definition(
        &self,
        params: request::GotoTypeDefinitionParams,
    ) -> Result<Option<request::GotoTypeDefinitionResponse>> {
        self.send_request::<request::GotoTypeDefinition>(params)
            .await
    }

    async fn goto_implementation(
        &self,
        params: request::GotoImplementationParams,
    ) -> Result<Option<request::GotoImplementationResponse>> {
        self.send_request::<request::GotoImplementation>(params)
            .await
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        self.send_request::<request::References>(params).await
    }

    async fn prepare_call_hierarchy(
        &self,
        params: CallHierarchyPrepareParams,
    ) -> Result<Option<Vec<CallHierarchyItem>>> {
        self.send_request::<request::CallHierarchyPrepare>(params)
            .await
    }

    async fn incoming_calls(
        &self,
        params: CallHierarchyIncomingCallsParams,
    ) -> Result<Option<Vec<CallHierarchyIncomingCall>>> {
        self.send_request::<request::CallHierarchyIncomingCalls>(params)
            .await
    }

    async fn outgoing_calls(
        &self,
        params: CallHierarchyOutgoingCallsParams,
    ) -> Result<Option<Vec<CallHierarchyOutgoingCall>>> {
        self.send_request::<request::CallHierarchyOutgoingCalls>(params)
            .await
    }

    async fn prepare_type_hierarchy(
        &self,
        params: TypeHierarchyPrepareParams,
    ) -> Result<Option<Vec<TypeHierarchyItem>>> {
        self.send_request::<request::TypeHierarchyPrepare>(params)
            .await
    }

    async fn supertypes(
        &self,
        params: TypeHierarchySupertypesParams,
    ) -> Result<Option<Vec<TypeHierarchyItem>>> {
        self.send_request::<request::TypeHierarchySupertypes>(params)
            .await
    }

    async fn subtypes(
        &self,
        params: TypeHierarchySubtypesParams,
    ) -> Result<Option<Vec<TypeHierarchyItem>>> {
        self.send_request::<request::TypeHierarchySubtypes>(params)
            .await
    }

    async fn document_highlight(
        &self,
        params: DocumentHighlightParams,
    ) -> Result<Option<Vec<DocumentHighlight>>> {
        self.send_request::<request::DocumentHighlightRequest>(params)
            .await
    }

    async fn document_link(&self, params: DocumentLinkParams) -> Result<Option<Vec<DocumentLink>>> {
        self.send_request::<request::DocumentLinkRequest>(params)
            .await
    }

    async fn document_link_resolve(&self, params: DocumentLink) -> Result<DocumentLink> {
        self.send_request::<request::DocumentLinkResolve>(params)
            .await
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        self.send_request::<request::HoverRequest>(params).await
    }

    async fn code_lens(&self, params: CodeLensParams) -> Result<Option<Vec<CodeLens>>> {
        self.send_request::<request::CodeLensRequest>(params).await
    }

    async fn code_lens_resolve(&self, params: CodeLens) -> Result<CodeLens> {
        self.send_request::<request::CodeLensResolve>(params).await
    }

    async fn folding_range(&self, params: FoldingRangeParams) -> Result<Option<Vec<FoldingRange>>> {
        self.send_request::<request::FoldingRangeRequest>(params)
            .await
    }

    async fn selection_range(
        &self,
        params: SelectionRangeParams,
    ) -> Result<Option<Vec<SelectionRange>>> {
        self.send_request::<request::SelectionRangeRequest>(params)
            .await
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        self.send_request::<request::DocumentSymbolRequest>(params)
            .await
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        self.send_request::<request::SemanticTokensFullRequest>(params)
            .await
    }

    async fn semantic_tokens_full_delta(
        &self,
        params: SemanticTokensDeltaParams,
    ) -> Result<Option<SemanticTokensFullDeltaResult>> {
        self.send_request::<request::SemanticTokensFullDeltaRequest>(params)
            .await
    }

    async fn semantic_tokens_range(
        &self,
        params: SemanticTokensRangeParams,
    ) -> Result<Option<SemanticTokensRangeResult>> {
        self.send_request::<request::SemanticTokensRangeRequest>(params)
            .await
    }

    async fn inline_value(&self, params: InlineValueParams) -> Result<Option<Vec<InlineValue>>> {
        self.send_request::<request::InlineValueRequest>(params)
            .await
    }

    async fn inlay_hint(&self, params: InlayHintParams) -> Result<Option<Vec<InlayHint>>> {
        self.send_request::<request::InlayHintRequest>(params).await
    }

    async fn inlay_hint_resolve(&self, params: InlayHint) -> Result<InlayHint> {
        self.send_request::<request::InlayHintResolveRequest>(params)
            .await
    }

    async fn moniker(&self, params: MonikerParams) -> Result<Option<Vec<Moniker>>> {
        self.send_request::<request::MonikerRequest>(params).await
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        self.send_request::<request::Completion>(params).await
    }

    async fn completion_resolve(&self, params: CompletionItem) -> Result<CompletionItem> {
        self.send_request::<request::ResolveCompletionItem>(params)
            .await
    }

    async fn diagnostic(
        &self,
        params: DocumentDiagnosticParams,
    ) -> Result<DocumentDiagnosticReportResult> {
        self.send_request::<request::DocumentDiagnosticRequest>(params)
            .await
    }

    async fn workspace_diagnostic(
        &self,
        params: WorkspaceDiagnosticParams,
    ) -> Result<WorkspaceDiagnosticReportResult> {
        self.send_request::<request::WorkspaceDiagnosticRequest>(params)
            .await
    }

    async fn signature_help(&self, params: SignatureHelpParams) -> Result<Option<SignatureHelp>> {
        self.send_request::<request::SignatureHelpRequest>(params)
            .await
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        self.send_request::<request::CodeActionRequest>(params)
            .await
    }

    async fn code_action_resolve(&self, params: CodeAction) -> Result<CodeAction> {
        self.send_request::<request::CodeActionResolveRequest>(params)
            .await
    }

    async fn document_color(&self, params: DocumentColorParams) -> Result<Vec<ColorInformation>> {
        self.send_request::<request::DocumentColor>(params).await
    }

    async fn color_presentation(
        &self,
        params: ColorPresentationParams,
    ) -> Result<Vec<ColorPresentation>> {
        self.send_request::<request::ColorPresentationRequest>(params)
            .await
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        self.send_request::<request::Formatting>(params).await
    }

    async fn range_formatting(
        &self,
        params: DocumentRangeFormattingParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        self.send_request::<request::RangeFormatting>(params).await
    }

    async fn on_type_formatting(
        &self,
        params: DocumentOnTypeFormattingParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        self.send_request::<request::OnTypeFormatting>(params).await
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        self.send_request::<request::Rename>(params).await
    }

    async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Option<PrepareRenameResponse>> {
        self.send_request::<request::PrepareRenameRequest>(params)
            .await
    }

    async fn linked_editing_range(
        &self,
        params: LinkedEditingRangeParams,
    ) -> Result<Option<LinkedEditingRanges>> {
        self.send_request::<request::LinkedEditingRange>(params)
            .await
    }

    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<WorkspaceSymbolResponse>> {
        self.send_request::<request::WorkspaceSymbolRequest>(params)
            .await
    }

    async fn symbol_resolve(&self, params: WorkspaceSymbol) -> Result<WorkspaceSymbol> {
        self.send_request::<request::WorkspaceSymbolResolve>(params)
            .await
    }

    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        self.send_notification::<notification::DidChangeConfiguration>(params)
            .await
    }

    async fn did_change_workspace_folders(&self, params: DidChangeWorkspaceFoldersParams) {
        self.send_notification::<notification::DidChangeWorkspaceFolders>(params)
            .await
    }

    async fn will_create_files(&self, params: CreateFilesParams) -> Result<Option<WorkspaceEdit>> {
        self.send_request::<request::WillCreateFiles>(params).await
    }

    async fn did_create_files(&self, params: CreateFilesParams) {
        self.send_notification::<notification::DidCreateFiles>(params)
            .await
    }

    async fn will_rename_files(&self, params: RenameFilesParams) -> Result<Option<WorkspaceEdit>> {
        self.send_request::<request::WillRenameFiles>(params).await
    }

    async fn did_rename_files(&self, params: RenameFilesParams) {
        self.send_notification::<notification::DidRenameFiles>(params)
            .await
    }

    async fn will_delete_files(&self, params: DeleteFilesParams) -> Result<Option<WorkspaceEdit>> {
        self.send_request::<request::WillDeleteFiles>(params).await
    }

    async fn did_delete_files(&self, params: DeleteFilesParams) {
        self.send_notification::<notification::DidDeleteFiles>(params)
            .await
    }

    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        self.send_notification::<notification::DidChangeWatchedFiles>(params)
            .await
    }

    async fn execute_command(&self, params: ExecuteCommandParams) -> Result<Option<LSPAny>> {
        self.send_request::<request::ExecuteCommand>(params).await
    }
}

impl<S> LanguageClient for LsServer<S>
where
    S: Service<Request, Response = Option<Response>> + Clone + Send + Sync + 'static,
    S::Error: Into<jsonrpc::Error>,
    S::Future: Send + 'static,
{
    async fn register_capability(&self, registrations: Vec<Registration>) -> jsonrpc::Result<()> {
        self.send_request::<request::RegisterCapability>(RegistrationParams { registrations })
            .await
    }

    async fn unregister_capability(
        &self,
        unregisterations: Vec<Unregistration>,
    ) -> jsonrpc::Result<()> {
        self.send_request::<request::UnregisterCapability>(UnregistrationParams {
            unregisterations,
        })
        .await
    }

    async fn show_message<M: Display + Send>(&self, typ: MessageType, message: M) {
        self.send_notification::<notification::ShowMessage>(ShowMessageParams {
            typ,
            message: message.to_string(),
        })
        .await;
    }

    async fn show_message_request<M: Display + Send>(
        &self,
        typ: MessageType,
        message: M,
        actions: Option<Vec<MessageActionItem>>,
    ) -> jsonrpc::Result<Option<MessageActionItem>> {
        self.send_request::<request::ShowMessageRequest>(ShowMessageRequestParams {
            typ,
            message: message.to_string(),
            actions,
        })
        .await
    }

    async fn log_message<M: Display + Send>(&self, typ: MessageType, message: M) {
        self.send_notification::<notification::LogMessage>(LogMessageParams {
            typ,
            message: message.to_string(),
        })
        .await;
    }

    async fn show_document(&self, params: ShowDocumentParams) -> jsonrpc::Result<bool> {
        self.send_request::<request::ShowDocument>(params)
            .await
            .map(|res| res.success)
    }

    async fn telemetry_event<Ser: Serialize + Send>(&self, data: Ser) {
        match serde_json::to_value(data) {
            Err(e) => error!("invalid JSON in `telemetry/event` notification: {}", e),
            Ok(value) => {
                let value = match value {
                    LSPAny::Object(value) => OneOf::Left(value),
                    LSPAny::Array(value) => OneOf::Right(value),
                    value => OneOf::Right(vec![value]),
                };
                self.send_notification::<notification::TelemetryEvent>(value)
                    .await;
            }
        }
    }

    async fn code_lens_refresh(&self) -> jsonrpc::Result<()> {
        self.send_request::<request::CodeLensRefresh>(()).await
    }

    async fn semantic_tokens_refresh(&self) -> jsonrpc::Result<()> {
        self.send_request::<request::SemanticTokensRefresh>(())
            .await
    }

    async fn inline_value_refresh(&self) -> jsonrpc::Result<()> {
        self.send_request::<request::InlineValueRefreshRequest>(())
            .await
    }

    async fn inlay_hint_refresh(&self) -> jsonrpc::Result<()> {
        self.send_request::<request::InlayHintRefreshRequest>(())
            .await
    }

    async fn workspace_diagnostic_refresh(&self) -> jsonrpc::Result<()> {
        self.send_request::<request::WorkspaceDiagnosticRefresh>(())
            .await
    }

    async fn publish_diagnostics(&self, uri: Uri, diags: Vec<Diagnostic>, version: Option<i32>) {
        self.send_notification::<notification::PublishDiagnostics>(PublishDiagnosticsParams::new(
            uri, diags, version,
        ))
        .await;
    }

    async fn create_work_done_progress(&self, token: ProgressToken) -> jsonrpc::Result<()> {
        self.send_request::<request::WorkDoneProgressCreate>(WorkDoneProgressCreateParams { token })
            .await
    }

    async fn configuration(&self, items: Vec<ConfigurationItem>) -> jsonrpc::Result<Vec<LSPAny>> {
        self.send_request::<request::WorkspaceConfiguration>(ConfigurationParams { items })
            .await
    }

    async fn workspace_folders(&self) -> jsonrpc::Result<Option<Vec<WorkspaceFolder>>> {
        self.send_request::<request::WorkspaceFoldersRequest>(())
            .await
    }

    async fn apply_edit(&self, edit: WorkspaceEdit) -> jsonrpc::Result<ApplyWorkspaceEditResponse> {
        self.send_request::<request::ApplyWorkspaceEdit>(ApplyWorkspaceEditParams {
            edit,
            label: None,
        })
        .await
    }
}
