use camino::{Utf8Path, Utf8PathBuf};
use std::io::Write;
use syn::{parse_quote, Item, ItemFn, ItemMod};
use thiserror::Error;

mod abstr;
mod refin;
mod support;
mod util;

#[derive(Clone)]
pub struct Machine {
    pub items: Vec<Item>,
}

impl Machine {
    pub fn from_file(file: syn::File) -> Self {
        // TODO: resolve attributes
        Self { items: file.items }
    }

    pub fn from_module(module: ItemMod) -> Option<Self> {
        // TODO: resolve attributes etc.
        let Some(content) = module.content else {
            return None;
        };
        Some(Self { items: content.1 })
    }

    pub fn abstract_machine(&self) -> Result<Machine, Error> {
        let mut abstract_machine = self.clone();
        support::ssa::apply(&mut abstract_machine)?;
        abstr::apply(&mut abstract_machine)?;
        refin::apply(&mut abstract_machine)?;
        Ok(abstract_machine)
    }

    pub fn with_main_fn(mut self) -> Self {
        // add main function

        let main_fn: ItemFn = parse_quote!(
            fn main() {
                ::machine_check_exec::run::<refin::Input, refin::State, refin::Machine>()
            }
        );
        self.items.push(Item::Fn(main_fn));
        self
    }

    pub fn write_to_file(self, filename: &Utf8Path) -> Result<(), Error> {
        let mut machine_file = std::fs::File::create(filename)
            .map_err(|err| Error::OpenFile(filename.to_path_buf(), err))?;

        let syn_file = syn::File {
            shebang: None,
            attrs: vec![],
            items: self.items,
        };

        let pretty_machine = prettyplease::unparse(&syn_file);

        machine_file
            .write_all(pretty_machine.as_bytes())
            .map_err(|err| Error::WriteFile(filename.to_path_buf(), err))
    }
}

#[derive(thiserror::Error, Debug, Clone)]
#[error("{0}")]
pub(crate) struct MachineError(String);

#[derive(Debug, Error)]
pub enum Error {
    #[error("could not open file {0}")]
    OpenFile(Utf8PathBuf, #[source] std::io::Error),
    #[error("could not write to file {0}")]
    WriteFile(Utf8PathBuf, #[source] std::io::Error),
    #[error("machine conversion error: {0}")]
    Machine(String),
}

impl From<MachineError> for Error {
    fn from(value: MachineError) -> Self {
        Error::Machine(value.0)
    }
}
