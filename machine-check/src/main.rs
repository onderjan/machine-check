use anyhow::anyhow;
use machine_check_lib::{create_abstract_machine, write_machine};
use std::{env, fs::File, path::Path, thread};
use syn::{parse_quote, Item, ItemFn};

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
    let mut abstract_machine = create_abstract_machine(&concrete_machine)?;

    // add main function

    let main_fn: ItemFn = parse_quote!(
        fn main() {
            ::machine_check_exec_lib::run::<mark::Machine>()
        }
    );
    abstract_machine.items.push(Item::Fn(main_fn));

    write_machine(
        "abstract",
        &abstract_machine,
        "machine-check-exec/src/main.rs",
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
