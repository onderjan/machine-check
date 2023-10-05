use anyhow::anyhow;
use core::panic;
use machine_check_exec_prepare::Preparation;
use machine_check_lib::{create_abstract_machine, write_machine};
use std::{
    collections::HashMap,
    env,
    fs::File,
    path::Path,
    process::{Command, Stdio},
    thread,
};
use syn::{parse_quote, Item, ItemFn};

fn execute_machine() -> Result<(), anyhow::Error> {
    println!("Building the machine.");

    let preparation_string =
        match std::fs::read_to_string("./resources/exec-build/preparation.json") {
            Ok(s) => s,
            Err(err) => return Err(anyhow!("Could not read preparation file: {:#?}", err)),
        };

    let preparation: Preparation = serde_json::from_str(preparation_string.as_str())?;
    let mut args = vec![
        String::from("machine-check-exec-target/src/main.rs"),
        String::from("--edition=2021"),
        String::from("--error-format=json"),
        String::from("--json=artifacts"),
        String::from("--crate-type"),
        String::from("bin"),
        String::from("-C"),
        String::from("opt-level=3"),
        String::from("-C"),
        String::from("embed-bitcode=no"),
        String::from("-C"),
        String::from("strip=symbols"),
        String::from("--out-dir"),
        String::from("./gen_build"),
    ];
    args.extend(preparation.target_build_args);

    let build_output = Command::new("rustc")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap();

    if !build_output.status.success() {
        return Err(anyhow!("Build was not successful"));
    }

    let mut artifact_path: Option<String> = None;
    let stderr = String::from_utf8(build_output.stderr)?;
    for line in stderr.lines() {
        let hash_map: HashMap<String, String> = serde_json::from_str(line)?;
        if let (Some(artifact), Some(emit)) = (hash_map.get("artifact"), hash_map.get("emit")) {
            if emit == "link" {
                // this is the executable
                artifact_path = Some(artifact.clone());
            }
        }
    }
    let Some(artifact_path) = artifact_path else {
        panic!("Build generated no artifact");
    };

    // run the artifact
    println!("Executing the machine.");

    let exec_output = Command::new(artifact_path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .unwrap();

    if !exec_output.status.success() {
        return Err(anyhow!("Execution was not successful"));
    }

    Ok(())
}

fn work() -> Result<(), anyhow::Error> {
    let mut args = env::args();
    // skip executable arg
    args.next();

    let Some(btor2_filename) = args.next() else {
        return Err(anyhow!("Input filename not specified"));
    };

    println!("Creating a machine for Btor2 file '{}'.", btor2_filename);

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
            ::machine_check_exec::run::<mark::Machine>()
        }
    );
    abstract_machine.items.push(Item::Fn(main_fn));

    write_machine(
        "abstract",
        &abstract_machine,
        "machine-check-exec-target/src/main.rs",
    )?;

    execute_machine()?;

    Ok(())
}

fn main() {
    // hook panic to propagate child panic
    let orig_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // invoke the default handler and exit the process
        orig_hook(panic_info);
        std::process::exit(1);
    }));

    // increase stack size by introducing a child thread
    // normal stack size is not enough for large token trees
    let result = thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(work)
        .unwrap()
        .join()
        .unwrap();

    if let Err(err) = result {
        eprintln!("Error: {:#}", err);
    }
}
