use std::fs::File;

use crate::btor2::parse_btor2;

mod btor2;
mod generator;

fn pretty(item: proc_macro2::TokenStream) -> String {
    let item_clone = item.clone();
    let Ok(file) = syn::parse_file(&item.to_string()) else {
        return format!("UNPARSABLE: {}", item_clone);
    };

    prettyplease::unparse(&file)
}

fn main() {
    let file = File::open("examples/recount4.btor2").unwrap();
    let btor2 = parse_btor2(file).unwrap();
    let tokens = generator::generate(btor2).unwrap();

    println!("{}", pretty(tokens));
}
