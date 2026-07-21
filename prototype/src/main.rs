#![allow(unused)]
mod sql;
mod ast;
lalrpop_mod!(pub chain);

use crate::sql::Query;
use lalrpop_util::lalrpop_mod;
use crate::ast::Chain;

fn main() {
    let try_trim = "\"Hello World!\"";
    let trimmed = try_trim[1..try_trim.len()-1].to_string();
    println!("{trimmed}");

    let chain: Chain = chain::ChainParser::new().parse("select(\"*\")\n    .    from   ( \"Students\"     )     ").unwrap();
    println!("{chain:?}");
}
