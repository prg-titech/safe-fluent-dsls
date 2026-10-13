use std::borrow::Borrow;
use std::collections::HashSet;
use std::error::Error;
use std::fmt::Display;
use std::hash::Hash;

use tower::filter::Predicate;
use tower_lsp_server::jsonrpc::Request;

pub struct MethodPredicate<S> {
    methods: HashSet<S>,
}

impl<S> MethodPredicate<S> {
    pub fn new<I: IntoIterator<Item = S>>(iter: I) -> Self 
    where 
        S: Hash + Eq
    {
        Self {
            methods: iter.into_iter().collect()
        }
    }
}

impl<S> Predicate<Request> for MethodPredicate<S>
where
    S: Borrow<str> + Hash + Eq,
{
    type Request = Request;

    fn check(&mut self, request: Request) -> Result<Self::Request, tower::BoxError> {
        if self.methods.contains(request.method()) {
            Ok(request)
        } else {
            Err(MethodNotFoundError {
                method: request.method().into(),
            }
            .into())
        }
    }
}

#[derive(Debug)]
struct MethodNotFoundError {
    method: String,
}

impl Display for MethodNotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Method not found error: {}", self.method)
    }
}

impl Error for MethodNotFoundError {}
