use std::sync::{Arc, atomic::AtomicI64};

use tower_lsp_server::jsonrpc::Id;

pub mod log;
pub mod id_map;
pub mod server;
pub mod client;

#[derive(Debug, Clone, Default)]
pub struct UniqueIdIter {
    index: Arc<AtomicI64>,
}

impl UniqueIdIter {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Iterator for UniqueIdIter {
    type Item = Id;

    fn next(&mut self) -> Option<Self::Item> {
        let current_index = self.index.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Some(Id::Number(current_index))
    }
}