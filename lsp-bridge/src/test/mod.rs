use std::sync::Arc;

use dashmap::DashMap;
use futures::future::{self, Ready};
use rstest::fixture;
use serde_json::json;
use tower::Service;
use tower_lsp_server::jsonrpc::{Id, Request, Response};

use crate::transport::error::ExitedError;

mod bridge;
pub mod log;

#[derive(Default, Clone)]
pub struct DummyService {
    pub encountered_ids: Arc<DashMap<Option<Id>, usize>>,
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
pub fn dummy_service() -> DummyService {
    DummyService::default()
}

pub fn dummy_request(id: i64) -> Request {
    Request::build("custom/ping").id(Id::Number(id)).finish()
}

pub fn dummy_notification() -> Request {
    Request::build("custom/nothing").finish()
}
