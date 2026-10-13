use futures::{
    FutureExt, TryFutureExt,
    future::{self, BoxFuture},
};
use std::{
    collections::HashMap,
    convert::Infallible,
    fmt::{self, Debug, Formatter},
    marker::PhantomData,
    sync::Arc,
    task::{Context, Poll},
};
use tower::{Layer, Service, util::BoxService};
use tower_lsp_server::{
    jsonrpc::{Error, FromParams, IntoResponse, Method, Request, Response},
    ls_types::LSPAny,
};

pub struct DefaultingRouter<S, T, E = Infallible> {
    router: Router<S, E>,
    default: T,
}

impl<S, T, E> DefaultingRouter<S, T, E> {
    pub fn new(router: Router<S, E>, default: T) -> Self {
        Self { router, default }
    }
}

impl<S, T, E> Service<Request> for DefaultingRouter<S, T, E>
where
    T: Service<Request, Response = Option<Response>>,
    T::Error: Into<E> + 'static,
    T::Future: Send + 'static,
    E: Send + 'static,
{
    type Response = Option<Response>;
    type Error = E;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.default.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        if self.router.methods.contains_key(req.method()) {
            self.router.call(req)
        } else {
            self.default.call(req).map_err(Into::into).boxed()
        }
    }
}

/// A modular JSON-RPC 2.0 request router service.
pub struct Router<S, E = Infallible> {
    server: Arc<S>,
    methods: HashMap<&'static str, BoxService<Request, Option<Response>, E>>,
}

impl<S: Send + Sync + 'static, E> Router<S, E> {
    /// Creates a new `Router` with the given shared state.
    pub fn new(server: S) -> Self {
        Self {
            server: Arc::new(server),
            methods: HashMap::new(),
        }
    }

    /// Returns a reference to the inner server.
    pub fn inner(&self) -> &S {
        self.server.as_ref()
    }

    /// Registers a new RPC method which constructs a response with the given `callback`.
    ///
    /// The `layer` argument can be used to inject middleware into the method handler, if desired.
    pub fn method<P, R, F, L>(&mut self, name: &'static str, callback: F, layer: L) -> &mut Self
    where
        P: FromParams,
        R: IntoResponse,
        F: for<'a> Method<&'a S, P, R> + Clone + Send + Sync + 'static,
        L: Layer<MethodHandler<P, R, E>>,
        L::Service: Service<Request, Response = Option<Response>, Error = E> + Send + 'static,
        <L::Service as Service<Request>>::Future: Send + 'static,
    {
        let server = &self.server;
        self.methods.entry(name).or_insert_with(|| {
            let server = server.clone();
            let handler = MethodHandler::new(move |params| {
                let callback = callback.clone();
                let server = server.clone();
                async move { callback.invoke(&*server, params).await }
            });

            BoxService::new(layer.layer(handler))
        });

        self
    }
}

impl<S: Debug, E> Debug for Router<S, E> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Router")
            .field("server", &self.server)
            .field("methods", &self.methods.keys())
            .finish()
    }
}

impl<S, E: Send + 'static> Service<Request> for Router<S, E> {
    type Response = Option<Response>;
    type Error = E;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        if let Some(handler) = self.methods.get_mut(req.method()) {
            handler.call(req)
        } else {
            let (method, id, _) = req.into_parts();
            future::ok(id.map(|id| {
                let mut error = Error::method_not_found();
                error.data = Some(LSPAny::from(method));
                Response::from_error(id, error)
            }))
            .boxed()
        }
    }
}

/// Opaque JSON-RPC method handler.
pub struct MethodHandler<P, R, E> {
    f: Box<dyn Fn(P) -> BoxFuture<'static, R> + Send>,
    _marker: PhantomData<E>,
}

impl<P: FromParams, R: IntoResponse, E> MethodHandler<P, R, E> {
    fn new<F, Fut>(handler: F) -> Self
    where
        F: Fn(P) -> Fut + Send + 'static,
        Fut: Future<Output = R> + Send + 'static,
    {
        Self {
            f: Box::new(move |p| handler(p).boxed()),
            _marker: PhantomData,
        }
    }
}

impl<P, R, E> Service<Request> for MethodHandler<P, R, E>
where
    P: FromParams,
    R: IntoResponse,
    E: Send + 'static,
{
    type Response = Option<Response>;
    type Error = E;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let (_, id, params) = req.into_parts();

        match id {
            Some(_) if R::is_notification() => return future::ok(().into_response(id)).boxed(),
            None if !R::is_notification() => return future::ok(None).boxed(),
            _ => {}
        }

        let params = match P::from_params(params) {
            Ok(params) => params,
            Err(err) => return future::ok(id.map(|id| Response::from_error(id, err))).boxed(),
        };

        (self.f)(params)
            .map(move |r| Ok(r.into_response(id)))
            .boxed()
    }
}
