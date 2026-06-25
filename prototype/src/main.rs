#![allow(unused)]

use crate::sql::Query;

mod sql;

fn main() {
    let my_query: Query = Query::builder()
        .select("name")
        .select("birthday")
        .from("Students")
        .from("Students2")
        .where_("birthday >= 2000-01-01")
        .where_("name == 'Alex Meyer'")
        .build();
    println!("{my_query}");

    let simple_query: Query = Query::builder()
        .select("*")
        .from("Students")
        .build();
    println!();
    println!("{simple_query}");
}
