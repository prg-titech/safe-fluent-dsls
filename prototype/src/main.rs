#![allow(unused)]
mod ast;
mod sql;
lalrpop_mod!(pub chain);

use crate::ast::Chain;
use lalrpop_util::lalrpop_mod;

fn main() {
    let try_trim = "\"Hello World!\"";
    let trimmed = try_trim[1..try_trim.len() - 1].to_string();
    println!("{trimmed}");

    let chain: Chain = chain::ChainParser::new()
        .parse("select(\"*\")\n    .    from   ( \"Students\"     )     ")
        .unwrap();
    println!("{chain:?}");
}
