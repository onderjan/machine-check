use camino::{Utf8Path, Utf8PathBuf};
use clap::{ArgGroup, Args};
use log::{debug, info};
use machine_check_common::ExecResult;
use serde::{Deserialize, Serialize};

use crate::CheckError;

#[derive(Debug, Clone, Args)]
#[clap(group(ArgGroup::new("property-group")
.required(true)
.multiple(true)
.args(&["property", "inherent", "gui"]),
))]
pub struct Cli {
    /// Whether to show the Graphical User Interface.
    #[arg(short, long, conflicts_with("property"), conflicts_with("inherent"))]
    pub gui: bool,

    /// Computation Tree Logic property to verify.
    #[arg(long)]
    pub property: Option<String>,

    /// Whether to verify the inherent property instead of a supplied one.
    #[arg(long)]
    pub inherent: bool,

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

    // if no property is supplied, we will verify the inherent one

    let config = machine_check_compile::VerifyConfig {
        abstract_machine,
        machine_path: verify_args.machine_path,
        preparation_path: verify_args.preparation_path,
        batch: args.batch,
        gui: verify_args.gui,
        property: verify_args.property,
        verbose: args.verbose,
        use_decay: verify_args.use_decay,
    };
    let verify_result = machine_check_compile::verify(config);
    if let Err(machine_check_compile::Error::Gui) = verify_result {
        return Ok(());
    }

    let exec_result = verify_result?;

    // print interesting facts
    info!(
        "Used {} states and {} refinements.",
        exec_result.stats.num_final_states, exec_result.stats.num_refinements
    );
    // print conclusion or return exec error
    let conclusion = exec_result.result.map_err(CheckError::ExecError)?;
    info!("Reached conclusion: {}", conclusion);
    Ok(())
}

fn process_machine(system_path: &Utf8Path) -> Result<syn::File, CheckError> {
    debug!("Constructing machine from path {:?}.", &system_path);
    let machine_file = super::translate::translate(system_path)?;
    Ok(machine_file)
}
