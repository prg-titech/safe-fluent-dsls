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

#[cfg(test)]
mod test {
    use std::{collections::HashMap, sync::Arc};

    use dashmap::DashMap;
    use futures::future::{self, Ready};
    use rstest::{fixture, rstest};
    use serde_json::json;
    use tower::{Layer, Service};
    use tower_lsp_server::jsonrpc::{Id, Request, Response};

    use crate::{id_layer::map_id, transport::error::ExitedError};

    #[derive(Default, Clone)]
    struct DummyService {
        encountered_ids: Arc<DashMap<Option<Id>, usize>>,
    }

    impl Service<Request> for DummyService {
        type Response = Option<Response>;
        type Error = ExitedError;
        type Future = Ready<Result<Self::Response, Self::Error>>;

        fn poll_ready(
            &mut self,
            _: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), Self::Error>> {
            std::task::Poll::Ready(Ok(()))
        }

        fn call(&mut self, req: Request) -> Self::Future {
            if let Some(id) = req.id().cloned() {
                self.encountered_ids
                    .entry(Some(id.clone()))
                    .and_modify(|u| {
                        *u += 1;
                    })
                    .or_insert(1);
                future::ready(Ok(Some(Response::from_ok(id.clone(), json!({"id": id})))))
            } else {
                self.encountered_ids
                    .entry(None)
                    .and_modify(|u| {
                        *u += 1;
                    })
                    .or_insert(1);
                future::ready(Ok(None))
            }
        }
    }

    #[fixture]
    fn dummy_service() -> DummyService {
        DummyService::default()
    }

    fn dummy_request(id: i64) -> Request {
        Request::build("custom/ping").id(Id::Number(id)).finish()
    }

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
            Arc::unwrap_or_clone(dummy_service.encountered_ids)
                .into_iter()
                .collect()
        );
    }
}
