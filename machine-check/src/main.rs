use anyhow::anyhow;
use std::{env, fs::File, path::Path};

mod forward;
mod mark;
mod ssa;
mod write;

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

    let concrete_machine = ssa::transcribe(btor2rs::translate_file(btor2_file)?)?;

    write::write_machine(
        "concrete",
        &concrete_machine,
        "machine-check-exec/src/machine/concrete.rs",
    )?;

    let forward_machine = forward::transcribe(concrete_machine)?;

    write::write_machine(
        "forward",
        &forward_machine,
        "machine-check-exec/src/machine/forward.rs",
    )?;

    Ok(())
}

fn main() {
    if let Err(err) = work() {
        eprintln!("Error: {:#}", err);
    }
}
