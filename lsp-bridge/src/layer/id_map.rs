use std::sync::Arc;

use dashmap::DashMap;
use futures::{FutureExt, future::BoxFuture};
use tower::{Layer, Service};
use tower_lsp_server::jsonrpc::{Id, Request, Response};

pub fn map_id(prefix: &'static str) -> MapIdLayer {
    MapIdLayer { prefix }
}

#[derive(Clone)]
pub struct MapIdLayer {
    prefix: &'static str,
}

impl MapIdLayer {
    pub fn prefix(&self) -> &'static str {
        &self.prefix
    }
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

#[derive(Debug, Clone)]
pub struct MapIdService<S> {
    prefix: &'static str,
    original_ids: Arc<DashMap<Box<str>, Id>>,
    pub inner: S,
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
    S: Service<Request, Response = Option<Response>>,
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
        let request = self.map_request(req);
        let original_ids = self.original_ids.clone();
        self.inner
            .call(request)
            .map(|r| r.map(|response| response.map(|r| Self::map_response(original_ids, r))))
            .boxed()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::{DummyService, dummy_request, dummy_service};
    use rstest::rstest;
    use serde_json::json;
    use std::{collections::HashMap, sync::Arc};
    use tower::{Layer, Service};
    use tower_lsp_server::jsonrpc::{Id, Response};

    #[rstest]
    #[tokio::test(flavor = "current_thread")]
    async fn test_map_id(mut dummy_service: DummyService) {
        let mut map_id_service = map_id("test").layer(dummy_service.clone());

        assert_eq!(
            Response::from_ok(Id::Number(1), json!({"id": Id::Number(1)})),
            dummy_service.call(dummy_request(1)).await.unwrap().unwrap()
        );
        assert_eq!(
            Response::from_ok(Id::Number(1), json!({"id": Id::String("test:1".into())})),
            map_id_service
                .call(dummy_request(1))
                .await
                .unwrap()
                .unwrap()
        );

        assert_eq!(
            Response::from_ok(Id::Number(2), json!({"id": Id::String("test:2".into())})),
            map_id_service
                .call(dummy_request(2))
                .await
                .unwrap()
                .unwrap()
        );
        assert_eq!(
            Response::from_ok(Id::Number(2), json!({"id": Id::Number(2)})),
            dummy_service.call(dummy_request(2)).await.unwrap().unwrap()
        );
        assert_eq!(
            Response::from_ok(Id::Number(2), json!({"id": Id::Number(2)})),
            dummy_service.call(dummy_request(2)).await.unwrap().unwrap()
        );

        let expected_ids: HashMap<Option<Id>, usize> = [
            (Some(Id::Number(1)), 1),
            (Some(Id::String("test:1".into())), 1),
            (Some(Id::Number(2)), 2),
            (Some(Id::String("test:2".into())), 1),
        ]
        .into_iter()
        .collect();
        assert_eq!(
            expected_ids,
            Arc::unwrap_or_clone(dummy_service.encountered_requests)
                .into_iter()
                .map(|(k, v)| (k, v.len()))
                .collect()
        );
    }
}
