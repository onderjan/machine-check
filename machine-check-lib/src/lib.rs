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

pub use write::write_machine;
