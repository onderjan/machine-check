use anyhow::anyhow;
use machine_check_lib::{create_abstract_machine, write_machine};
use std::{env, fs::File, path::Path, thread};

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

    let concrete_machine: syn::File = syn::parse2(btor2rs::translate_file(btor2_file)?)?;
    let abstract_machine = create_abstract_machine(&concrete_machine)?;

    write_machine(
        "abstract",
        &abstract_machine,
        "machine-check-exec/src/machine.rs",
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
