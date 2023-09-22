use anyhow::anyhow;
use proc_macro2::TokenStream;
use std::{env, fs::File, io::Write, path::Path};

fn pretty_string(item: TokenStream) -> String {
    let str = item.to_string();
    if let Ok(file) = syn::parse_file(&str) {
        return prettyplease::unparse(&file);
    }
    eprintln!("Warning: could not parse token stream to format it");
    item.to_string()
}

fn work() -> Result<(), anyhow::Error> {
    let mut args = env::args();
    // skip executable arg
    args.next();

    let Some(btor2_filename) = args.next() else {
        return Err(anyhow!("Input filename not specified"));
    };

    println!("Input filename: {}", btor2_filename);

    let btor2_file = match File::open(Path::new(&btor2_filename)) {
        Ok(file) => file,
        Err(err) => {
            return Err(anyhow!(
                "Cannot open input file '{}': {}",
                btor2_filename,
                err
            ))
        }
    };

    let concrete_machine = btor2rs::translate_file(btor2_file)?;

    let concrete_machine_path = Path::new("machine-check-exec/src/machine/concrete.rs");
    if !concrete_machine_path.exists() {
        return Err(anyhow!(
            "Concrete machine file to be replaced does not exist"
        ));
    }
    let mut concrete_machine_file = match File::options()
        .write(true)
        .truncate(true)
        .open(concrete_machine_path)
    {
        Ok(file) => file,
        Err(err) => return Err(anyhow!("Cannot open concrete machine file: {}", err)),
    };

    if let Err(err) = concrete_machine_file.write(pretty_string(concrete_machine).as_bytes()) {
        return Err(anyhow!("Cannot write concrete machine to file: {}", err));
    }
    println!(
        "Written concrete machine to {}",
        concrete_machine_path.display()
    );

    Ok(())
}

fn main() {
    if let Err(err) = work() {
        eprintln!("Error: {:#}", err);
    }
}
