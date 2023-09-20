use std::fs::File;

use crate::btor2::parse_btor2;

mod btor2;
mod generator;

fn pretty(item: proc_macro2::TokenStream) -> String {
    let str = item.to_string();
    let Ok(file) = syn::parse_file(&str) else {
        return format!("/* Unparsable */ {}", item);
    };
    prettyplease::unparse(&file)
}

fn main() {
    let file = File::open("examples/recount4.btor2").unwrap();
    let btor2 = parse_btor2(file).unwrap();
    let tokens = generator::generate(btor2).unwrap();

    println!("{}", pretty(tokens));
}
