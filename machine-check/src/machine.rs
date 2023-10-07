use camino::Utf8Path;
use std::io::Write;
use syn::File;

use crate::CheckError;

mod transcription;
mod write;

pub(crate) fn create_abstract_machine(concrete_machine: &File) -> anyhow::Result<File> {
    let mut abstract_machine = concrete_machine.clone();
    transcription::manipulation::ssa::apply(&mut abstract_machine)?;
    transcription::abstraction::forward::apply(&mut abstract_machine)?;
    transcription::abstraction::mark::apply(&mut abstract_machine)?;
    transcription::manipulation::field_manipulation::apply(&mut abstract_machine)?;
    Ok(abstract_machine)
}

pub(crate) fn write_machine(machine: &syn::File, filename: &Utf8Path) -> Result<(), CheckError> {
    let mut machine_file = std::fs::File::create(filename)
        .map_err(|err| CheckError::OpenFile(filename.to_path_buf(), err))?;

    let pretty_machine = prettyplease::unparse(machine);

    machine_file
        .write_all(pretty_machine.as_bytes())
        .map_err(|err| CheckError::WriteFile(filename.to_path_buf(), err))
}
