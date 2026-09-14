use std::fmt::{self, Display, Formatter};
use std::io::Error as IoError;
use std::num::ParseIntError;
use std::str::Utf8Error;

use tower_lsp_server::ls_types::Uri;

#[derive(Debug, Clone)]
pub enum Error {
    TreeSitterParserError,
    FileAlreadyExists{uri: Uri},
    TexterError(texter::error::Error)
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::TreeSitterParserError => write!(f, "An error occurred during parsing"),
            Error::FileAlreadyExists { uri } => write!(f, "File '{}' already exists", uri.as_str()),
            Error::TexterError(error) => write!(f, "Texter error: {error}")
        }
    }
}

impl std::error::Error for Error {}

impl From<texter::error::Error> for Error {
    fn from(value: texter::error::Error) -> Self {
        Self::TexterError(value)
    }
}

/// Errors that can occur when processing an LSP message.
#[derive(Debug)]
pub enum ParseError {
    /// Failed to parse the JSON body.
    Body(serde_json::Error),
    /// Failed to encode the response.
    Encode(IoError),
    /// Failed to parse headers.
    Headers(httparse::Error),
    /// The media type in the `Content-Type` header is invalid.
    InvalidContentType,
    /// The length value in the `Content-Length` header is invalid.
    InvalidContentLength(ParseIntError),
    /// Request lacks the required `Content-Length` header.
    MissingContentLength,
    /// Request contains invalid UTF8.
    Utf8(Utf8Error),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Self::Body(ref e) => write!(f, "unable to parse JSON body: {e}"),
            Self::Encode(ref e) => write!(f, "failed to encode response: {e}"),
            Self::Headers(ref e) => write!(f, "failed to parse headers: {e}"),
            Self::InvalidContentType => write!(f, "unable to parse content type"),
            Self::InvalidContentLength(ref e) => {
                write!(f, "unable to parse content length: {e}")
            }
            Self::MissingContentLength => {
                write!(f, "missing required `Content-Length` header")
            }
            Self::Utf8(ref e) => write!(f, "request contains invalid UTF8: {e}"),
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Self::Body(ref e) => Some(e),
            Self::Encode(ref e) => Some(e),
            Self::InvalidContentLength(ref e) => Some(e),
            Self::Utf8(ref e) => Some(e),
            _ => None,
        }
    }
}

impl From<serde_json::Error> for ParseError {
    fn from(error: serde_json::Error) -> Self {
        Self::Body(error)
    }
}

impl From<IoError> for ParseError {
    fn from(error: IoError) -> Self {
        Self::Encode(error)
    }
}

impl From<httparse::Error> for ParseError {
    fn from(error: httparse::Error) -> Self {
        Self::Headers(error)
    }
}

impl From<ParseIntError> for ParseError {
    fn from(error: ParseIntError) -> Self {
        Self::InvalidContentLength(error)
    }
}

impl From<Utf8Error> for ParseError {
    fn from(error: Utf8Error) -> Self {
        Self::Utf8(error)
    }
}