use std::path::PathBuf;

use proc_macro2::{Ident, Span};
use syn::{parse_quote, Item, ItemFn, ItemMod};
use thiserror::Error;

use crate::util::create_item_mod;

mod abstr;
mod refin;
mod ssa;
mod support;
mod util;

#[derive(Clone)]
pub struct MachineDescription {
    pub items: Vec<Item>,
}

pub fn process_file(mut file: syn::File) -> Result<syn::File, Error> {
    process_items(&mut file.items)?;
    Ok(file)
}

pub fn process_module(mut module: ItemMod) -> Result<ItemMod, Error> {
    let Some((_, items)) = &mut module.content else {
        return Err(Error::Machine(String::from(
            "Cannot process module without content",
        )));
    };
    process_items(items)?;
    Ok(module)
}

pub fn default_main() -> Item {
    let main_fn: ItemFn = parse_quote!(
        fn main() {
            ::machine_check_exec::run::<refin::Input, refin::State, refin::Machine>()
        }
    );
    Item::Fn(main_fn)
}

fn out_dir() -> PathBuf {
    let mut args = std::env::args();

    let mut out_dir = None;
    while let Some(arg) = args.next() {
        if arg == "--out-dir" {
            out_dir = args.next();
        }
    }
    // find target directory
    let mut out_dir = PathBuf::from(out_dir.expect("Failed to find out_dir"));
    while !out_dir.ends_with("target") {
        if !out_dir.pop() {
            panic!("Cannot find out_dir");
        }
    }

    out_dir
}

fn unparse(machine: &MachineDescription) -> String {
    prettyplease::unparse(&syn::File {
        shebang: None,
        attrs: vec![],
        items: machine.items.clone(),
    })
}

fn process_items(items: &mut Vec<Item>) -> Result<(), Error> {
    println!("Machine-check-machine starting processing");

    let out_dir = out_dir();

    let ssa_machine = ssa::create_concrete_machine(items.clone())?;
    std::fs::write(out_dir.join("machine_ssa.rs"), unparse(&ssa_machine))
        .expect("SSA machine file should be writable");

    let mut abstract_machine = abstr::create_abstract_machine(&ssa_machine)?;

    std::fs::write(out_dir.join("machine_abstr.rs"), unparse(&abstract_machine))
        .expect("Abstract machine file should be writable");

    let refinement_machine = refin::create_refinement_machine(&abstract_machine)?;

    // create new module at the end of the file that will contain the refinement
    let refinement_module = create_machine_module("refin", refinement_machine);
    abstract_machine.items.push(refinement_module);

    std::fs::write(out_dir.join("machine_full.rs"), unparse(&abstract_machine))
        .expect("Full machine file should be writable");

    support::strip_machine::strip_machine(&mut abstract_machine)?;

    *items = abstract_machine.items;

    println!("Machine-check-machine ending processing");

    Ok(())
}

fn create_machine_module(name: &str, machine: MachineDescription) -> Item {
    Item::Mod(create_item_mod(
        syn::Visibility::Public(Default::default()),
        Ident::new(name, Span::call_site()),
        machine.items,
    ))
}

#[derive(thiserror::Error, Debug, Clone)]
#[error("{0}")]
pub(crate) struct MachineError(String);

#[derive(Debug, Error)]
pub enum Error {
    #[error("machine conversion error: {0}")]
    Machine(String),
}

impl From<MachineError> for Error {
    fn from(value: MachineError) -> Self {
        Error::Machine(value.0)
    }
}
