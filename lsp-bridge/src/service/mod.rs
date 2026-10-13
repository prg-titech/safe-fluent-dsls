pub mod either;
pub mod filter;

use tower::{Service, filter::Predicate};

use crate::service::either::Either;

pub trait ServiceExt<Request>: Service<Request> {
    fn filter_or<F, S, NewRequest>(self, filter: F, other: S) -> Either<Self, S, F>
    where
        Self: Sized,
        F: Predicate<NewRequest>,
    {
        Either::new(self, other, filter)
    }
}

impl<T, Request> ServiceExt<Request> for T where T: Service<Request> {}
