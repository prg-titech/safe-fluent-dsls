use std::task::ready;

use futures::FutureExt;
use futures::TryFutureExt;
use futures::future::BoxFuture;
use tower::{BoxError, Service, filter::Predicate};

pub struct Either<S1, S2, F> {
    predicate: F,
    left: S1,
    right: S2,
}

impl<S1, S2, F> Either<S1, S2, F> {
    pub fn new(left: S1, right: S2, predicate: F) -> Self {
        Self {
            predicate,
            left,
            right,
        }
    }
}

impl<S1, S2, F, NewRequest> Service<NewRequest> for Either<S1, S2, F>
where
    NewRequest: Clone,
    F: Predicate<NewRequest>,
    S1: Service<F::Request>,
    S1::Error: Into<BoxError> + 'static,
    S1::Future: Send + 'static,
    S2: Service<NewRequest, Response = S1::Response>,
    S2::Error: Into<BoxError> + 'static,
    S2::Future: Send + 'static,
{
    type Response = S1::Response;
    type Error = BoxError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        ready!(self.left.poll_ready(cx)).map_err(Into::into)?;
        self.right.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: NewRequest) -> Self::Future {
        match self.predicate.check(req.clone()) {
            Ok(request) => self.left.call(request).map_err(Into::into).boxed(),
            Err(_) => self.right.call(req).map_err(Into::into).boxed(),
        }
    }
}
