use std::sync::Arc;

use dashmap::DashMap;
use futures::{FutureExt, future::BoxFuture};
use tower::{Layer, Service};
use tower_lsp_server::jsonrpc::{Id, Request, Response};

use crate::transport::core::LsService;

pub fn map_id(prefix: &'static str) -> MapIdLayer {
    MapIdLayer { prefix }
}

pub struct MapIdLayer {
    prefix: &'static str,
}

impl<S> Layer<S> for MapIdLayer {
    type Service = MapIdService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        MapIdService {
            prefix: self.prefix,
            original_ids: Arc::default(),
            inner,
        }
    }
}

pub struct MapIdService<S> {
    prefix: &'static str,
    original_ids: Arc<DashMap<Box<str>, Id>>,
    inner: S,
}

impl<S> MapIdService<S> {
    fn map_request(&self, request: Request) -> Request {
        let (method, id, body) = request.into_parts();
        let id = id.map(|id| self.add_prefix(id));
        let mut result = Request::build(method);
        if let Some(id) = id {
            result = result.id(id);
        }
        if let Some(params) = body {
            result = result.params(params);
        }
        result.finish()
    }

    fn add_prefix(&self, id: Id) -> Id {
        let with_prefix = match id.clone() {
            Id::Null => {
                return id;
            }
            Id::Number(i) => format!("{}:{i}", self.prefix),
            Id::String(s) => format!("{}:{s}", self.prefix),
        };
        self.original_ids
            .insert(with_prefix.clone().into_boxed_str(), id);
        Id::String(with_prefix)
    }

    fn map_response(original_ids: Arc<DashMap<Box<str>, Id>>, response: Response) -> Response {
        let (id, body) = response.into_parts();
        let id = match id {
            Id::Null => Id::Null,
            Id::Number(_) => unreachable!(),
            Id::String(s) => Self::remove_prefix(original_ids, &s),
        };
        Response::from_parts(id, body)
    }

    fn remove_prefix(original_ids: Arc<DashMap<Box<str>, Id>>, s: &str) -> Id {
        original_ids.remove(s).unwrap().1
    }
}

impl<S> Service<Request> for MapIdService<S>
where
    S: LsService,
    S::Future: Send,
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
        let request = self.map_request(req);
        let original_ids = self.original_ids.clone();
        self.inner
            .call(request)
            .map(|r| r.map(|response| response.map(|r| Self::map_response(original_ids, r))))
            .boxed()
    }
}