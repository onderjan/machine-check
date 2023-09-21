use std::{ffi::OsStr, fs::File, path::Path, time::Instant};

use proc_macro2::TokenStream;
use walkdir::WalkDir;

#[allow(dead_code)]
fn generate_complex_machines() {
    println!(
        "Current dir: {}",
        std::env::current_dir().unwrap().display()
    );
    let mut num_ok: usize = 0;
    let mut num_err: usize = 0;

    let mut current_ok = 0;

    let start = Instant::now();

    for entry in WalkDir::new("btor2rs/examples/complex") {
        let entry = entry.expect("Should be able to walk");
        let path = entry.path();
        let extension = path.extension().and_then(OsStr::to_str);
        if let Some("btor2") = extension {
            let file = File::open(path).expect("Should be able to open a file");
            let result = btor2rs::translate_file(file);
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
    }

    let duration = start.elapsed();

    println!(
        "Processed all complex examples, {} ok, {} errors, took {:?}",
        num_ok, num_err, duration
    );
}

#[allow(dead_code)]
fn pretty(item: TokenStream) -> String {
    let str = item.to_string();
    let Ok(file) = syn::parse_file(&str) else {
        return format!("/* Unparsable */ {}", item);
    };
    prettyplease::unparse(&file)
}

#[allow(dead_code)]
fn generate_normal_machine() {
    let filename = "btor2rs/examples/recount4.btor2";
    //let filename = "btor2rs/examples/easy_zero_array.btor2";
    let file = File::open(Path::new(filename)).expect("Should be able to open Btor2 file");
    let tokens = btor2rs::translate_file(file).expect("Should be able to translate Btor2 file");
    println!("Normal machine result:");
    println!();
    println!("{}", pretty(tokens));
}

fn main() {
    //generate_complex_machines();
    generate_normal_machine();
}
