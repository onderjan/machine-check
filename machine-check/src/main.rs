use anyhow::anyhow;
use std::{env, fs::File, path::Path, thread};

mod transcription;
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

    let mut concrete_machine: syn::File = syn::parse2(btor2rs::translate_file(btor2_file)?)?;

    transcription::simplification::ssa::apply(&mut concrete_machine)?;

    write::write_machine(
        "concrete",
        &concrete_machine,
        "machine-check-exec/src/machine/concrete.rs",
    )?;

    let mut forward_machine = concrete_machine.clone();
    transcription::abstraction::forward::apply(&mut forward_machine)?;
    transcription::abstraction::mark::apply(&mut forward_machine)?;

    write::write_machine(
        "forward",
        &forward_machine,
        "machine-check-exec/src/machine/forward.rs",
    )?;

    Ok(())
}

fn main() {
    // increase stack size by introducing a child thread
    // normal stack size is not enough for large token trees
    thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(move || {
            if let Err(err) = work() {
                eprintln!("Error: {:#}", err);
            }
        })
        .unwrap()
        .join()
        .unwrap();
}
