use camino::{Utf8Path, Utf8PathBuf};
use std::io::Write;
use syn::File;
use thiserror::Error;

mod abstr;
mod refin;
mod support;
mod util;

pub fn create_abstract_machine(concrete_machine: &File) -> Result<File, Error> {
    let mut abstract_machine = concrete_machine.clone();
    support::ssa::apply(&mut abstract_machine)?;
    abstr::apply(&mut abstract_machine)?;
    refin::apply(&mut abstract_machine)?;
    Ok(abstract_machine)
}

pub fn write_machine(machine: &syn::File, filename: &Utf8Path) -> Result<(), Error> {
    let mut machine_file = std::fs::File::create(filename)
        .map_err(|err| Error::OpenFile(filename.to_path_buf(), err))?;

    let pretty_machine = prettyplease::unparse(machine);

    machine_file
        .write_all(pretty_machine.as_bytes())
        .map_err(|err| Error::WriteFile(filename.to_path_buf(), err))
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
