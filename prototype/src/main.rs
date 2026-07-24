#![allow(unused)]
mod ast;
mod sql;
mod chain;

use crate::ast::Chain;
use crate::chain::ChainParser;

use tree_sitter::{InputEdit, Language, Parser, Point};

fn main() {
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_java::LANGUAGE.into()).expect("Error loading Java grammar");

    let source_code =
r#"public class Example {
    public void main(String[] args) {
        Query query = SQL.select("*").from("Students").where("age >= 18").build();
    }
}"#;
    println!("{source_code}");

    let mut tree = parser.parse(source_code, None).unwrap();
    let root_node = tree.root_node();
    println!("{root_node:#?}");
}
