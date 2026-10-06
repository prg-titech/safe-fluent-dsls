#![allow(async_fn_in_trait)]

use std::fmt::Display;

use log::error;
use serde::Serialize;
use tower_lsp_server::{jsonrpc, ls_types::*};

pub trait LanguageHandle {
    fn send_request<R>(
        &self,
        params: R::Params,
    ) -> impl Future<Output = jsonrpc::Result<R::Result>> + Send
    where
        R: request::Request;

    fn send_notification<N>(&self, params: N::Params) -> impl Future<Output = ()> + Send
    where
        N: notification::Notification;
}

pub trait LanguageClient: LanguageHandle {
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

    async fn show_message<M: Display>(&self, typ: MessageType, message: M) {
        self.send_notification::<notification::ShowMessage>(ShowMessageParams {
            typ,
            message: message.to_string(),
        })
        .await;
    }

    async fn show_message_request<M: Display>(
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

    async fn log_message<M: Display>(&self, typ: MessageType, message: M) {
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

    async fn telemetry_event<S: Serialize>(&self, data: S) {
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

impl<T: LanguageHandle> LanguageClient for T {}
