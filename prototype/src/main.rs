#![allow(unused)]
mod ast;
mod sql;
mod chain;

use crate::ast::Chain;
use crate::chain::ChainParser;

fn main() {
    let chain: Chain = ChainParser::new()
        .parse(r#"select("*").from("Students")"#)
        .unwrap();
    println!("{chain}");
}
