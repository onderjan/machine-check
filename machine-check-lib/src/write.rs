use anyhow::anyhow;
use std::io::Write;
use std::{fs::File, path::Path};

pub fn write_machine(
    machine_type: &str,
    machine: &syn::File,
    filename: &str,
) -> Result<(), anyhow::Error> {
    let machine_path: &Path = Path::new(filename);
    let mut machine_file = match File::options()
        .write(true)
        .truncate(true)
        .open(machine_path)
    {
        Ok(file) => file,
        Err(err) => {
            return Err(anyhow!(
                "Cannot open {} machine file '{}': {}",
                machine_type,
                filename,
                err
            ))
        }
    };

    let pretty_machine = prettyplease::unparse(machine);

    if let Err(err) = machine_file.write_all(pretty_machine.as_bytes()) {
        return Err(anyhow!(
            "Cannot write {} machine to file '{}': {}",
            machine_type,
            filename,
            err
        ));
    }
    Ok(())
}
