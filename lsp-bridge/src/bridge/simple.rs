use crate::bridge::Bridge;

#[derive(Clone)]
pub struct SimpleBridge<S> {
    inner: Bridge<S>
}

