use std::fmt::Display;

use futures::{FutureExt, future::BoxFuture};
use log::{error, info};
use serde::Serialize;
use tower::{Layer, Service};
use tower_lsp_server::jsonrpc::Response;

#[derive(Debug, Default, Clone, Copy)]
pub struct LogLayer {
    source: &'static str,
    target: &'static str,
    reversed: bool,
}

impl LogLayer {
    pub fn new(source: &'static str, target: &'static str, reversed: bool) -> Self {
        Self {
            source,
            target,
            reversed,
        }
    }
}

impl LogLayer {
    fn log_serializable<Ser: Serialize>(
        &self,
        prefix: &str,
        left_to_right: bool,
        serializable: &Ser,
    ) {
        let arrow = if left_to_right { "-->" } else { "<--" };
        match serde_json::to_string(serializable) {
            Ok(s) => info!("{prefix} : {} {arrow} {} {s}", self.source, self.target),
            Err(e) => error!("{prefix} : {} {arrow} {} {e}", self.source, self.target),
        }
    }
}

impl<S> Layer<S> for LogLayer {
    type Service = LogService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LogService {
            inner,
            layer: *self,
        }
    }
}

#[derive(Clone)]
pub struct LogService<S> {
    inner: S,
    layer: LogLayer,
}

impl<S, Request> Service<Request> for LogService<S>
where
    Request: Serialize,
    S: Service<Request, Response = Option<Response>> + Send + Sync,
    S::Error: Display + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Option<Response>;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let layer = self.layer;
        layer.log_serializable("Request ", !layer.reversed, &req);
        self.inner
            .call(req)
            .then(async move |r| {
                match &r {
                    Ok(Some(r)) => {
                        layer.log_serializable("Response", layer.reversed, r);
                    }
                    Ok(None) => (),
                    Err(e) => {
                        error!("Error:    {e}")
                    }
                };
                r
            })
            .boxed()
    }
}

#[cfg(test)]
mod test {
    use std::io::{BufRead, BufReader};

    use rstest::{fixture, rstest};
    use simplelog::{ConfigBuilder, WriteLogger};
    use tower::{Layer, Service};

    use crate::{logging::LogLayer, test::{DummyService, dummy_notification, dummy_request, dummy_service}};

    #[fixture]
    fn test_log() -> impl BufRead {
        let (log_tx, log_rx) = std::io::pipe().unwrap();
        WriteLogger::init(
            log::LevelFilter::Trace,
            ConfigBuilder::new().build(),
            log_rx,
        )
        .unwrap();
        BufReader::new(log_tx)
    }

    #[rstest]
    #[tokio::test]
    async fn test_log_service(test_log: impl BufRead, mut dummy_service: DummyService) {
        let mut full_log = test_log.lines();
        let notification = dummy_notification();
        assert_eq!(Ok(None), dummy_service.call(notification.clone()).await);

        let mut log_service = LogLayer::new("client", "server", false).layer(dummy_service.clone());
        assert_eq!(Ok(None), log_service.call(notification.clone()).await);
        let line = full_log.next().unwrap().unwrap();
        assert!(line.contains("INFO"));
        assert!(line.contains("Request"));
        assert!(line.contains("client --> server"));
        assert!(line.contains(serde_json::to_string(&notification).unwrap().as_str()));

        let request = dummy_request(1);
        let response = log_service.call(request.clone()).await.unwrap().unwrap();
        let request_line = full_log.next().unwrap().unwrap();
        assert!(request_line.contains("INFO"));
        assert!(request_line.contains("Request"));
        assert!(request_line.contains("client --> server"));
        assert!(request_line.contains(serde_json::to_string(&request).unwrap().as_str()));

        let response_line = full_log.next().unwrap().unwrap();
        assert!(response_line.contains("INFO"));
        assert!(response_line.contains("Response"));
        assert!(response_line.contains("client <-- server"));
        assert!(response_line.contains(serde_json::to_string(&response).unwrap().as_str()));

        let mut log_service = LogLayer::new("client", "server", true).layer(dummy_service.clone());
        assert_eq!(Ok(None), log_service.call(notification.clone()).await);
        let line = full_log.next().unwrap().unwrap();
        assert!(line.contains("INFO"));
        assert!(line.contains("Request"));
        assert!(line.contains("client <-- server"));
        assert!(line.contains(serde_json::to_string(&notification).unwrap().as_str()));

        let request = dummy_request(1);
        let response = log_service.call(request.clone()).await.unwrap().unwrap();
        let request_line = full_log.next().unwrap().unwrap();
        assert!(request_line.contains("INFO"));
        assert!(request_line.contains("Request"));
        assert!(request_line.contains("client <-- server"));
        assert!(request_line.contains(serde_json::to_string(&request).unwrap().as_str()));

        let response_line = full_log.next().unwrap().unwrap();
        assert!(response_line.contains("INFO"));
        assert!(response_line.contains("Response"));
        assert!(response_line.contains("client --> server"));
        assert!(response_line.contains(serde_json::to_string(&response).unwrap().as_str()));
    }
}
