use std::fmt::Display;

use futures::{FutureExt, future::BoxFuture};
use log::{error, info};
use serde::Serialize;
use tower::{Layer, Service};

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

pub struct LogService<S> {
    inner: S,
    layer: LogLayer,
}

impl<S, Request> Service<Request> for LogService<S>
where
    Request: Serialize,
    S: Service<Request> + Send + Sync,
    S::Response: Serialize + Send + 'static,
    S::Error: Display + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
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
                    Ok(r) => {
                        layer.log_serializable("Response", layer.reversed, r);
                    }
                    Err(e) => {
                        error!("Error:    {e}")
                    }
                };
                r
            })
            .boxed()
    }
}
