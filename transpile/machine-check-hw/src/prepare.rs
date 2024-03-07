use cargo_metadata::camino::Utf8PathBuf;
use clap::Args;

use crate::CheckError;

#[derive(Debug, Clone, Args)]
pub struct Cli {
    /// Location of preparation directory, defaults to "machine-check-preparation" next to the executable.
    #[arg(long)]
    pub preparation_path: Option<Utf8PathBuf>,

    /// Remove the preparation directory and its content instead of creating/updating it.
    #[arg(long)]
    pub clean: bool,
}

pub(crate) fn prepare(_: super::Cli, prepare: Cli) -> Result<(), CheckError> {
    machine_check_compile::prepare(machine_check_compile::PrepareConfig {
        preparation_path: prepare.preparation_path,
        clean: prepare.clean,
    })?;
    Ok(())
}
