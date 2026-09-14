use std::{collections::VecDeque, fmt::Debug, io::Write, marker::PhantomData, path::PathBuf, process::Stdio};

use assert_cmd::cargo::cargo_bin;
use dashmap::DashMap;
use escargot::CargoBuild;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tokio::{io::{AsyncRead, AsyncWrite}, process::{Child, ChildStdin, ChildStdout, Command}};
use tower_lsp_server::jsonrpc::{Id, Request, Response};
use tokio_util::{bytes::{Buf, BufMut, BytesMut}, codec::{Decoder, Encoder, FramedRead, FramedWrite}};
use tracing::{trace, warn};
use memchr::memmem;
use futures::{SinkExt, StreamExt};
use crate::error::ParseError;


const fn number_of_digits(mut n: usize) -> usize {
    let mut num_digits = 0;

    while n > 0 {
        n /= 10;
        num_digits += 1;
    }

    num_digits
}

/// An incoming or outgoing JSON-RPC message.
#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum Message {
    /// A response message.
    Response(Response),
    /// A request or notification message.
    Request(Request),
}

/// Encodes and decodes Language Server Protocol messages.
pub struct LanguageServerCodec<T> {
    content_len: Option<usize>,
    _marker: PhantomData<T>,
}

impl<T> Default for LanguageServerCodec<T> {
    fn default() -> Self {
        Self {
            content_len: None,
            _marker: PhantomData,
        }
    }
}

impl<T: Serialize> Encoder<T> for LanguageServerCodec<T> {
    type Error = ParseError;

    fn encode(&mut self, item: T, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let msg = serde_json::to_string(&item)?;
        trace!("-> {}", msg);

        // Reserve just enough space to hold the `Content-Length: ` and `\r\n\r\n` constants,
        // the length of the message, and the message body.
        dst.reserve(msg.len() + number_of_digits(msg.len()) + 20);
        let mut writer = dst.writer();
        write!(writer, "Content-Length: {}\r\n\r\n{}", msg.len(), msg)?;
        writer.flush()?;

        Ok(())
    }
}

impl<T: DeserializeOwned> Decoder for LanguageServerCodec<T> {
    type Item = T;
    type Error = ParseError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if let Some(content_len) = self.content_len {
            if src.len() < content_len {
                return Ok(None);
            }

            let bytes = &src[..content_len];
            let message = std::str::from_utf8(bytes)?;

            let result = if message.is_empty() {
                Ok(None)
            } else {
                trace!("<- {}", message);
                match serde_json::from_str(message) {
                    Ok(parsed) => Ok(Some(parsed)),
                    Err(err) => Err(err.into()),
                }
            };

            src.advance(content_len);
            self.content_len = None; // Reset state in preparation for parsing next message.

            result
        } else {
            let mut dst = [httparse::EMPTY_HEADER; 2];

            let (headers_len, headers) = match httparse::parse_headers(src, &mut dst)? {
                httparse::Status::Complete(output) => output,
                httparse::Status::Partial => return Ok(None),
            };

            match decode_headers(headers) {
                Ok(content_len) => {
                    src.advance(headers_len);
                    self.content_len = Some(content_len);
                    self.decode(src) // Recurse right back in, now that `Content-Length` is known.
                }
                Err(err) => {
                    match err {
                        ParseError::MissingContentLength => {}
                        _ => src.advance(headers_len),
                    }

                    // Skip any garbage bytes by scanning ahead for another potential message.
                    src.advance(memmem::find(src, b"Content-Length").unwrap_or_default());
                    Err(err)
                }
            }
        }
    }
}

fn decode_headers(headers: &[httparse::Header<'_>]) -> Result<usize, ParseError> {
    let mut content_len = None;

    for header in headers {
        match header.name {
            "Content-Length" => {
                let string = std::str::from_utf8(header.value)?;
                let parsed_len = string.parse()?;
                content_len = Some(parsed_len);
            }
            "Content-Type" => {
                let string = std::str::from_utf8(header.value)?;
                let charset = string
                    .split(';')
                    .skip(1)
                    .map(str::trim)
                    .find_map(|param| param.strip_prefix("charset="));

                match charset {
                    Some("utf-8" | "utf8") => {}
                    _ => return Err(ParseError::InvalidContentType),
                }
            }
            other => warn!("encountered unsupported header: {:?}", other),
        }
    }

    content_len.ok_or(ParseError::MissingContentLength)
}

pub struct TestClient<I, O, D> {
    pub child: Child,
    input: FramedRead<I, D>,
    output: FramedWrite<O, D>,
    pending_requests: VecDeque<Request>,
    responses: DashMap<Id, Response>,
    notifications: VecDeque<Response>,
}

impl<I, O, D> TestClient<I, O, D> 
where
    I: AsyncRead + Unpin,
    O: AsyncWrite + Unpin,
    D: Encoder<Message> + Decoder<Item=Message>,
    <D as Encoder<Message>>::Error: Debug,
    <D as Decoder>::Error: Debug,
{
    fn build_binary(curr_dir: &PathBuf) -> Result<(), std::io::Error> {
        let result = CargoBuild::new()
            .bin("sqls")
            .run()
            .expect("Failed to build server");

        result.command().current_dir(curr_dir).spawn()?;
        Ok(())
    }

    fn spawn_binary(curr_dir: &PathBuf) -> Result<Child, std::io::Error> {
        let bin = cargo_bin("sqls");

        Command::new(bin)
            .current_dir(curr_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }

    pub async fn make_request(&mut self, request: Request) -> Response {
        let id = request.id().expect("Expected an id").clone();
        if id == Id::Null {
            panic!("Expected a non-null Id");
        }
        self.output.send(Message::Request(request)).await.unwrap();
        self.wait_for_response(id).await
    }

    pub async fn send_notification(&mut self, notification: Request) {
        self.output.send(Message::Request(notification)).await.unwrap();
    }

    pub async fn receive_request(&mut self) -> Request {
        while self.pending_requests.is_empty() {
            self.wait_for_messages(1).await;
        }
        self.pending_requests.pop_front().unwrap()
    }

    async fn wait_for_response(&mut self, id: Id) -> Response {
        while !self.responses.contains_key(&id) {
            self.wait_for_messages(1).await;
        }

        self.responses.remove(&id).unwrap().1
    }

    async fn wait_for_messages(&mut self, n: usize) {
        for _ in 0..n {
            let message = self.input.next().await.expect("Input unexpectedly closed").expect("Input parse error");
            match message {
                Message::Request(request) => self.pending_requests.push_back(request),
                Message::Response(response) => {
                    if response.is_error() || response.id() == &Id::Null {
                        self.notifications.push_back(response);
                    } else {
                        self.responses.insert(response.id().clone(), response);
                    }
                }
            }
        }
    }
}

pub async fn stdio() -> TestClient<ChildStdout, ChildStdin, LanguageServerCodec<Message>> {
    let curr_dir = std::env::current_dir().expect("Failed to fetch current directory");

    TestClient::<ChildStdout, ChildStdin, LanguageServerCodec<Message>>::build_binary(&curr_dir).expect("Failed to build sqls binary");
    let mut child = TestClient::<ChildStdout, ChildStdin, LanguageServerCodec<Message>>::spawn_binary(&curr_dir).expect("Failed to start sqls");

    let stdin = child.stdout.take().expect("Failed to open stdin");
    let framed_stdin = FramedRead::new(stdin, LanguageServerCodec::default());
    let stdout = child.stdin.take().expect("Failed to open stdout");
    let framed_stdout = FramedWrite::new(stdout, LanguageServerCodec::default());


    TestClient { 
        child: child, 
        input: framed_stdin, 
        output: framed_stdout, 
        pending_requests: VecDeque::new(), 
        responses: DashMap::new(), 
        notifications: VecDeque::new()
    }
}

pub type DefaultTestClient = TestClient<ChildStdout, ChildStdin, LanguageServerCodec<Message>>;