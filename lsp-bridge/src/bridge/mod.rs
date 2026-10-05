pub mod core;

use lazy_static::lazy_static;
use std::collections::HashSet;

lazy_static! {
    static ref METHODS: HashSet<&'static str> = {
        [
            "initialize",
            "initialized",
            "shutdown",
            "textDocument/didOpen",
            "textDocument/didChange",
            "textDocument/didClose",
        ]
        .into_iter()
        .collect()
    };
}