use proc_macro2::TokenStream;
use std::{ffi::OsStr, fs::File, path::Path, time::Instant};
use walkdir::WalkDir;

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

fn generate_machine(path: &Path) -> Result<TokenStream, anyhow::Error> {
    let file = File::open(path)?;
    let btor2 = parse_btor2(file)?;
    generator::generate(btor2)
}

fn generate_complex_machines() {
    let mut num_ok: usize = 0;
    let mut num_err: usize = 0;

    let mut current_ok = 0;

    let start = Instant::now();

    for entry in WalkDir::new("examples/complex") {
        let entry = entry.expect("Should be able to walk");
        let path = entry.path();
        let extension = path.extension().and_then(OsStr::to_str);
        if let Some("btor2") = extension {
            let result = generate_machine(path);
            if (current_ok != 0 && result.is_err()) || current_ok == 100 {
                println!("(... {} OK ...)", current_ok);
                current_ok = 0;
            }
            match result {
                Ok(_) => {
                    num_ok += 1;
                    current_ok += 1;
                }
                Err(err) => {
                    println!("ERROR [{}]: {:#}", path.display(), err);
                    num_err += 1;
                }
            }
        }
    }

    if current_ok != 0 {
        println!("(... {} OK ...)", current_ok);
        current_ok = 0;
    }

    let duration = start.elapsed();

    println!(
        "Processed all complex examples, {} ok, {} errors, took {:?}",
        num_ok, num_err, duration
    );
}

fn generate_normal_machine() {
    let result = generate_machine(Path::new("examples/easy_zero_array.btor2"));
    match result {
        Ok(tokens) => {
            println!("Normal machine result:");
            println!();
            println!("{}", pretty(tokens));
        }
        Err(err) => eprintln!("Error generating normal machine: {}", err),
    }
}

fn main() {
    generate_complex_machines();
}
