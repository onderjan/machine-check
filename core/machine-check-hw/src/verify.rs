use camino::{Utf8Path, Utf8PathBuf};
use clap::Args;
use log::{debug, info};
use machine_check_common::ExecResult;
use serde::{Deserialize, Serialize};

use crate::CheckError;

#[derive(Debug, Clone, Args)]
pub struct Cli {
    /// Computation Tree Logic property to verify. Defaults to specification within the system.
    #[arg(long)]
    pub property: Option<String>,

    /// Where the machine crate should be created. Defaults to temporary directory.
    #[arg(long)]
    pub machine_path: Option<Utf8PathBuf>,

    /// Location of preparation directory. Defaults to "machine-check-preparation" next to the executable.
    #[arg(long)]
    pub preparation_path: Option<Utf8PathBuf>,

    /// Location of the system to verify.
    pub system_path: Utf8PathBuf,

    /// Whether state decay should be used. This can speed up or slow down verification depending on the system.
    #[arg(long)]
    pub use_decay: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyResult {
    pub exec: Option<ExecResult>,
    pub stats: VerifyStats,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyStats {
    pub transcription_time: Option<f64>,
    pub build_time: Option<f64>,
    pub execution_time: Option<f64>,
    pub prepared: Option<bool>,
}

pub(crate) fn run(args: super::Cli, verify_args: Cli) -> Result<(), CheckError> {
    let abstract_machine = process_machine(&verify_args.system_path)?;

    let config = machine_check_compile::VerifyConfig {
        abstract_machine,
        machine_path: verify_args.machine_path,
        preparation_path: verify_args.preparation_path,
        batch: args.batch,
        property: verify_args.property,
        verbose: args.verbose,
        use_decay: verify_args.use_decay,
    };
    let exec_result = machine_check_compile::verify(config)?;

    // print interesting facts
    info!(
        "Used {} states and {} refinements.",
        exec_result.stats.num_states, exec_result.stats.num_refinements
    );
    // print conclusion or return exec error
    let conclusion = exec_result.result?;
    info!("Reached conclusion: {}", conclusion);
    Ok(())
}

fn process_machine(system_path: &Utf8Path) -> Result<syn::File, CheckError> {
    debug!("Constructing machine from path {:?}.", &system_path);
    let machine_file = super::translate::translate(system_path)?;
    let mut processed_file = machine_check_machine::process_file(machine_file)
        .map_err(|err| CheckError::Machine(err.to_string()))?;
    processed_file
        .items
        .push(machine_check_machine::default_main());
    Ok(processed_file)
}
