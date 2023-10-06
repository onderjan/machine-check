use anyhow::anyhow;
use camino::Utf8Path;
use std::fs::File;
use std::io::Write;

pub fn write_machine(
    machine_type: &str,
    machine: &syn::File,
    filename: &Utf8Path,
) -> Result<(), anyhow::Error> {
    let mut machine_file = match File::create(filename) {
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
