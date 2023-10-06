use anyhow::anyhow;
use camino::Utf8Path;
use std::io::Write;
use syn::File;

mod transcription;
mod write;

pub fn create_abstract_machine(concrete_machine: &File) -> anyhow::Result<File> {
    let mut abstract_machine = concrete_machine.clone();
    transcription::manipulation::ssa::apply(&mut abstract_machine)?;
    transcription::abstraction::forward::apply(&mut abstract_machine)?;
    transcription::abstraction::mark::apply(&mut abstract_machine)?;
    transcription::manipulation::field_manipulation::apply(&mut abstract_machine)?;
    Ok(abstract_machine)
}

pub fn write_machine(
    machine_type: &str,
    machine: &syn::File,
    filename: &Utf8Path,
) -> Result<(), anyhow::Error> {
    let mut machine_file = match std::fs::File::create(filename) {
        Ok(file) => file,
        Err(err) => {
            return Err(anyhow!(
                "Cannot open {} machine file {:?}: {}",
                machine_type,
                filename,
                err
            ))
        }
    };

    let pretty_machine = prettyplease::unparse(machine);

    if let Err(err) = machine_file.write_all(pretty_machine.as_bytes()) {
        return Err(anyhow!(
            "Cannot write {} machine to file '{:?}': {}",
            machine_type,
            filename,
            err
        ));
    }
    Ok(())
}
