use anyhow::anyhow;
use std::io::Write;
use std::{fs::File, path::Path};

use proc_macro2::TokenStream;

fn pretty_string(item: &TokenStream) -> String {
    let str = item.to_string();
    if let Ok(file) = syn::parse_file(&str) {
        return prettyplease::unparse(&file);
    }
    eprintln!("Warning: could not parse token stream to format it");
    item.to_string()
}

pub fn write_machine(
    machine_type: &str,
    machine: &TokenStream,
    filename: &str,
) -> Result<(), anyhow::Error> {
    let machine_path: &Path = Path::new(filename);
    let mut machine_file = match File::options()
        .write(true)
        .truncate(true)
        .open(machine_path)
    {
        Ok(file) => file,
        Err(err) => {
            return Err(anyhow!(
                "Cannot open {} machine file '{}': {}",
                machine_type,
                filename,
                err
            ))
        }
    };

    if let Err(err) = machine_file.write(pretty_string(machine).as_bytes()) {
        return Err(anyhow!(
            "Cannot write {} machine to file '{}': {}",
            machine_type,
            filename,
            err
        ));
    }
    println!("Written {} machine", machine_type);
    Ok(())
}
