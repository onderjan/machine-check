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

fn process_items(items: &mut Vec<Item>) -> Result<(), Error> {
    let ssa_machine = ssa::create_concrete_machine(items.clone())?;
    /*println!(
        "SSA machine: {}",
        prettyplease::unparse(&syn::File {
            shebang: None,
            attrs: vec![],
            items: ssa_machine.items.clone()
        })
    );*/
    let mut abstract_machine = abstr::create_abstract_machine(&ssa_machine)?;
    let refinement_machine = refin::create_refinement_machine(&abstract_machine)?;

    // create new module at the end of the file that will contain the refinement
    let refinement_module = create_machine_module("refin", refinement_machine);
    abstract_machine.items.push(refinement_module);

    /*println!(
        "Processed items before stripping: {}",
        prettyplease::unparse(&syn::File {
            shebang: None,
            attrs: vec![],
            items: abstract_machine.items.clone()
        })
    );*/

    support::strip_machine::strip_machine(&mut abstract_machine)?;

    *items = abstract_machine.items;

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
