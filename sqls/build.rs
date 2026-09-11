use std::path::PathBuf;

use auto_lsp_codegen::generate;
use tree_sitter_sequel::{NODE_TYPES, LANGUAGE};
use std::fs;
use std::collections::HashMap;

fn main() {
    let output_path = PathBuf::from("src/generated.rs");

    let token_map: HashMap<&'static str, &'static str> = [
        ("`", "grave_accent")
    ].into_iter().collect();

    let generated = generate(NODE_TYPES, &LANGUAGE.into(), Some(token_map));
    let ast = &syn::parse2(generated).unwrap();


    fs::write(
        output_path,
        prettyplease::unparse(ast)
    ).unwrap();
}