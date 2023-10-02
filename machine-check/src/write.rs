use anyhow::anyhow;
use quote::ToTokens;
use std::io::{BufWriter, Write};
use std::{fs::File, path::Path};
use syn::token;

pub fn write_machine(
    machine_type: &str,
    machine: &syn::File,
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

    println!("Converting to token stream");
    // do not use prettyplease as it can overflow the stack
    //let unparsed = prettyplease::unparse(machine);
    let token_stream = machine.to_token_stream();

    println!("Writing to file");
    let mut writer = BufWriter::new(&machine_file);
    if let Err(err) = write!(writer, "{}", token_stream) {
        return Err(anyhow!(
            "Cannot write {} machine to file '{}': {}",
            machine_type,
            filename,
            err
        ));
    }
    if let Err(err) = writer.flush() {
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
