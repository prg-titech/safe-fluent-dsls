use std::{error::Error, fmt::{self, Display, Formatter}};

use futures::channel::{mpsc, oneshot};
use tower_lsp_server::jsonrpc::{self, Id};

/// Error that occurs when attempting to call the language server after it has already exited.
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ExitedError;

impl Error for ExitedError {}

impl Display for ExitedError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("language server has exited")
    }
}

impl From<ExitedError> for jsonrpc::Error {
    fn from(_: ExitedError) -> Self {
        Self {
            code: jsonrpc::ErrorCode::ServerError(-333444),
            message: "language server has exited".into(),
            data: None,
        }
    }
}

impl From<SendError> for ExitedError {
    fn from(value: SendError) -> Self {
        match value {
            SendError::Disconnected => ExitedError,
            SendError::Full => unreachable!("Socket is full despite waiting until free"),
            SendError::UnknownId(_) => unreachable!(),
        }
    }
}

impl From<mpsc::SendError> for ExitedError {
    fn from(_: mpsc::SendError) -> Self {
        ExitedError
    }
}

impl From<oneshot::Canceled> for ExitedError {
    fn from(_: oneshot::Canceled) -> Self {
        ExitedError
    }
}

impl From<tower_lsp_server::ExitedError> for ExitedError {
    fn from(_: tower_lsp_server::ExitedError) -> Self {
        ExitedError
    }
}

#[derive(Debug, Clone)]
pub enum SendError {
    Full,
    Disconnected,
    UnknownId(Id),
}

impl Display for SendError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "SendError: ")
    }
}

impl Error for SendError {}

impl From<futures::channel::mpsc::SendError> for SendError {
    fn from(value: futures::channel::mpsc::SendError) -> Self {
        if value.is_full() {
            SendError::Full
        } else {
            SendError::Disconnected
        }
    }
}

impl<T> From<mpsc::TrySendError<T>> for SendError {
    fn from(value: mpsc::TrySendError<T>) -> Self {
        if value.is_full() {
            SendError::Full
        } else {
            SendError::Disconnected
        }
    }
}

impl From<tower_lsp_server::ExitedError> for SendError {
    fn from(_: tower_lsp_server::ExitedError) -> Self {
        SendError::Disconnected
    }
}
