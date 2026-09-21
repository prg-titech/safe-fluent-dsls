use std::{collections::HashMap, pin::Pin};

use futures::FutureExt;
use tower::{Service, util::BoxService};
use tower_lsp_server::jsonrpc::{Request, Response};

use crate::transport::ExitedError;

pub type BoxLsService = BoxService<Request, Option<Response>, ExitedError>;

pub struct Router<D> {
    custom_routes: HashMap<&'static str, BoxLsService>,
    default_route: D,
}

impl<D> Router<D> {
    pub fn new(default_route: D) -> Self {
        Self {
            custom_routes: HashMap::new(),
            default_route,
        }
    }

    pub fn register_route_boxed(&mut self, method: &'static str, service: BoxLsService) {
        self.custom_routes
            .insert(method, service)
            .expect(&format!("Registered method {method} twice"));
    }
}

impl<D> Service<Request> for Router<D>
where
    D: Service<Request, Response = Option<Response>, Error = ExitedError>,
    D::Future: Future + Send + 'static,
{
    type Response = Option<Response>;
    type Error = ExitedError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        match self.custom_routes.get_mut(req.method()) {
            Some(service) => service.call(req),
            None => self.default_route.call(req).boxed(),
        }
    }
}
