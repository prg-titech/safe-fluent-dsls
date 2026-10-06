use std::{
    sync::{Arc, RwLock, TryLockError},
    task::{Context, Poll},
};

use futures::future;
use log::error;
use tower::{Layer, Service};

#[derive(Clone, Default)]
pub struct ShareableLayer;

impl<S> Layer<S> for ShareableLayer {
    type Service = ShareableService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ShareableService {
            inner: Arc::new(RwLock::new(inner)),
        }
    }
}

pub struct ShareableService<S> {
    inner: Arc<RwLock<S>>,
}

impl<S> Clone for ShareableService<S> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}

impl<S> ShareableService<S> {
    pub async fn wait_and_call<Request>(&self, request: Request) -> Result<S::Response, S::Error>
    where
        S: Service<Request>,
        S::Error: Default,
    {
        let mut cloned = self.clone();
        future::poll_fn(|cx| cloned.poll_ready(cx)).await?;
        cloned.call(request).await
    }
}

impl<S, Request> Service<Request> for ShareableService<S>
where
    S: Service<Request>,
    S::Error: Default,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match self.inner.try_write() {
            Ok(mut g) => g.poll_ready(cx),
            Err(TryLockError::WouldBlock) => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Err(TryLockError::Poisoned(e)) => {
                error!("Read-Write Lock of shareable service was poisoned with error: {e}");
                Poll::Ready(Err(Self::Error::default()))
            }
        }
    }

    fn call(&mut self, req: Request) -> Self::Future {
        self.inner
            .try_write()
            .expect(
                "Failed to acquire write lock of shareable service. Did you poll_ready() before?",
            )
            .call(req)
    }
}
