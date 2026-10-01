use futures::{
    FutureExt, Sink, SinkExt, Stream, StreamExt,
    future::{self, Join},
};
use std::{ops::{Deref, DerefMut}, pin::Pin};
use tower::{Layer, Service, ServiceExt, util::BoxService};
use tower_lsp_server::jsonrpc::{Request, Response};

pub struct Schedulable<T, F> {
    inner: T,
    task: F,
}

pub type Task<T> = Pin<Box<dyn Future<Output = T> + Send + 'static>>;

impl<T, F> Schedulable<T, F> {
    pub fn new(inner: T, task: F) -> Self {
        Self { inner, task }
    }

    pub fn into_parts(self) -> (T, F) {
        let Self { inner, task } = self;
        (inner, task)
    }

    pub fn schedule_with<S, O>(self, spawn: S) -> T
    where
        S: FnOnce(F) -> O,
    {
        let (inner, task) = self.into_parts();
        let _ = spawn(task);
        inner
    }

    pub fn schedule(self) -> T
    where
        F: Future + Send + 'static,
        F::Output: Send + Sync,
    {
        self.schedule_with(tokio::spawn)
    }

    pub async fn serve(self) -> (T, F::Output)
    where
        F: Future,
    {
        let (inner, task) = self.into_parts();
        let task_result = task.await;
        (inner, task_result)
    }

    pub fn map<U, Fun>(self, f: Fun) -> Schedulable<U, F>
    where
        Fun: FnOnce(T) -> U,
    {
        let (inner, task) = self.into_parts();
        Schedulable::new(f(inner), task)
    }

    pub fn map_task<Fun, U>(self, f: Fun) -> Schedulable<T, Task<U>>
    where
        F: Future + Send + 'static,
        Fun: FnOnce(F::Output) -> U + Send + Sync + 'static,
    {
        let (inner, task) = self.into_parts();
        Schedulable {
            inner,
            task: task.map(|o| f(o)).boxed(),
        }
    }

    pub fn then<U, Fun, Fut>(self, f: Fun) -> Schedulable<U, Join<F, Fut>>
    where
        F: Future,
        Fun: FnOnce(T) -> Schedulable<U, Fut>,
        Fut: Future,
    {
        let (inner, task) = self.into_parts();
        let (result, task2) = f(inner).into_parts();
        let combined_task = future::join(task, task2);
        Schedulable::new(result, combined_task)
    }

    pub fn join<U, G>(self, other: Schedulable<U, G>) -> Schedulable<(T, U), Join<F, G>>
    where
        F: Future,
        G: Future,
    {
        let (t, f) = self.into_parts();
        let (u, g) = other.into_parts();
        Schedulable {
            inner: (t, u),
            task: future::join(f, g),
        }
    }
}

pub trait LsService:
    Service<Request, Response = Option<Response>> + Send + Sync + Unpin + 'static
{
}

impl<T> LsService for T where
    T: Service<Request, Response = Option<Response>> + Send + Sync + Unpin + 'static
{
}

pub trait LsSocket {
    type RequestStream: Stream<Item = Request> + Unpin + Send + 'static;
    type ResponseSink: Sink<Response> + Unpin + Send + 'static;

    fn split(self) -> (Self::RequestStream, Self::ResponseSink);

    fn connect<S: LsService>(self, mut service: S) -> Schedulable<(), Task<()>>
    where
        Self: Sized,
        S::Future: Send,
        S::Error: Send,
    {
        let (mut request_rx, mut response_tx) = self.split();
        let handle_requests = async move {
            while let Some(request) = request_rx.next().await {
                match service.call(request).await {
                    Ok(Some(response)) => {
                        let _ = response_tx.send(response).await;
                    }
                    Ok(None) => (),
                    Err(_) => {
                        break;
                    }
                }
            }
        }
        .boxed();
        Schedulable {
            inner: (),
            task: handle_requests,
        }
    }
}

pub struct LsHandle<Service, Socket> {
    service: Service,
    socket: Socket,
}

pub type BoxLsService<E> = BoxService<Request, Option<Response>, E>;

impl<Service, Socket> LsHandle<Service, Socket> {
    pub fn new(service: Service, socket: Socket) -> Self {
        Self { service, socket }
    }

    pub fn layer<L: Layer<Service>>(self, layer: L) -> LsHandle<L::Service, Socket> {
        let Self { service, socket } = self;
        LsHandle {
            service: layer.layer(service),
            socket,
        }
    }

    pub fn boxed(self) -> LsHandle<BoxLsService<Service::Error>, Socket>
    where
        Service: LsService,
        Service::Future: Send,
    {
        let Self { service, socket } = self;
        LsHandle {
            service: service.boxed(),
            socket,
        }
    }

    pub fn split(self) -> (Service, Socket) {
        let Self { service, socket } = self;
        (service, socket)
    }
}

impl<Se, So> Deref for LsHandle<Se, So> {
    type Target = Se;

    fn deref(&self) -> &Self::Target {
        &self.service
    }
}

impl<Se, So> DerefMut for LsHandle<Se, So> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.service
    }
}
