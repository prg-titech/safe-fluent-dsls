use std::{fmt::Display, future};

use log::error;
use serde::Serialize;
use tower_lsp_server::{
    jsonrpc::{self, Error},
    ls_types::*,
};

pub trait LanguageHandle: Send + Sync + 'static {
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

pub trait LanguageClient: Send + Sync + 'static {
    fn register_capability(
        &self,
        registrations: Vec<Registration>,
    ) -> impl Future<Output = jsonrpc::Result<()>> + Send {
        let _ = registrations;
        error!("Client does not implement method client/registerCapability");
        future::ready(Err(Error::method_not_found()))
    }

    fn unregister_capability(
        &self,
        unregisterations: Vec<Unregistration>,
    ) -> impl Future<Output = jsonrpc::Result<()>> + Send {
        let _ = unregisterations;
        error!("Client does not implement method client/unregisterCapability");
        future::ready(Err(Error::method_not_found()))
    }

    fn show_message<M: Display + Send>(
        &self,
        typ: MessageType,
        message: M,
    ) -> impl Future<Output = ()> + Send {
        let _ = typ;
        let _ = message;
        error!("Client does not implement method window/showMessage");
        future::ready(())
    }

    fn show_message_request<M: Display + Send>(
        &self,
        typ: MessageType,
        message: M,
        actions: Option<Vec<MessageActionItem>>,
    ) -> impl Future<Output = jsonrpc::Result<Option<MessageActionItem>>> + Send {
        let _ = typ;
        let _ = message;
        let _ = actions;
        error!("Client does not implement method window/showMessageRequest");
        future::ready(Err(Error::method_not_found()))
    }

    fn log_message<M: Display + Send>(
        &self,
        typ: MessageType,
        message: M,
    ) -> impl Future<Output = ()> + Send {
        let _ = typ;
        let _ = message;
        error!("Client does not implement method window/logMessage");
        future::ready(())
    }

    fn show_document(
        &self,
        params: ShowDocumentParams,
    ) -> impl Future<Output = jsonrpc::Result<bool>> + Send {
        let _ = params;
        error!("Client does not implement method window/showDocument");
        future::ready(Err(Error::method_not_found()))
    }

    fn telemetry_event<S: Serialize + Send>(&self, data: S) -> impl Future<Output = ()> + Send {
        let _ = data;
        error!("Client does not implement method telemetry/event");
        future::ready(())
    }

    fn code_lens_refresh(&self) -> impl Future<Output = jsonrpc::Result<()>> + Send {
        error!("Client does not implement method workspace/codeLens/refresh");
        future::ready(Err(Error::method_not_found()))
    }

    fn semantic_tokens_refresh(&self) -> impl Future<Output = jsonrpc::Result<()>> + Send {
        error!("Client does not implement method workspace/semanticTokens/refresh");
        future::ready(Err(Error::method_not_found()))
    }

    fn inline_value_refresh(&self) -> impl Future<Output = jsonrpc::Result<()>> + Send {
        error!("Client does not implement method workspace/inlineValue/refresh");
        future::ready(Err(Error::method_not_found()))
    }

    fn inlay_hint_refresh(&self) -> impl Future<Output = jsonrpc::Result<()>> + Send {
        error!("Client does not implement method workspace/inlayHint/refresh");
        future::ready(Err(Error::method_not_found()))
    }

    fn workspace_diagnostic_refresh(&self) -> impl Future<Output = jsonrpc::Result<()>> + Send {
        error!("Client does not implement method workspace/diagnostic/refresh");
        future::ready(Err(Error::method_not_found()))
    }

    fn publish_diagnostics(
        &self,
        uri: Uri,
        diags: Vec<Diagnostic>,
        version: Option<i32>,
    ) -> impl Future<Output = ()> + Send {
        let _ = uri;
        let _ = diags;
        let _ = version;
        error!("Client does not implement method textDocument/publishDiagnostics");
        future::ready(())
    }

    fn create_work_done_progress(
        &self,
        token: ProgressToken,
    ) -> impl Future<Output = jsonrpc::Result<()>> + Send {
        let _ = token;
        error!("Client does not implement method window/workDoneProgress/create");
        future::ready(Err(Error::method_not_found()))
    }

    fn configuration(
        &self,
        items: Vec<ConfigurationItem>,
    ) -> impl Future<Output = jsonrpc::Result<Vec<LSPAny>>> + Send {
        let _ = items;
        error!("Client does not implement method workspace/configuration");
        future::ready(Err(Error::method_not_found()))
    }

    fn workspace_folders(
        &self,
    ) -> impl Future<Output = jsonrpc::Result<Option<Vec<WorkspaceFolder>>>> + Send {
        error!("Client does not implement method workspace/workspaceFolders");
        future::ready(Err(Error::method_not_found()))
    }

    fn apply_edit(
        &self,
        edit: WorkspaceEdit,
    ) -> impl Future<Output = jsonrpc::Result<ApplyWorkspaceEditResponse>> + Send {
        let _ = edit;
        error!("Client does not implement method workspace/applyEdit");
        future::ready(Err(Error::method_not_found()))
    }
}
