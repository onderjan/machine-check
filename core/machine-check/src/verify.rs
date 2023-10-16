use camino::Utf8PathBuf;
use clap::Args;
use log::info;
use machine_check_common::ExecResult;
use serde::{Deserialize, Serialize};

use crate::CheckError;

mod build;
mod execute;
mod translate;
mod work;

#[derive(Debug, Clone, Args)]
pub struct Cli {
    #[arg(long)]
    pub property: Option<String>,

    #[arg(long)]
    pub output_path: Option<Utf8PathBuf>,

    #[arg(long)]
    pub machine_path: Option<Utf8PathBuf>,

    #[arg(long)]
    pub preparation_path: Option<Utf8PathBuf>,

    pub system_path: Utf8PathBuf,

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
    let mut verify = Verify {
        args,
        verify_args,
        stats: VerifyStats {
            transcription_time: None,
            build_time: None,
            execution_time: None,
            prepared: None,
        },
    };

    let exec_result = verify.work();

    let exec = match &exec_result {
        Ok(ok) => Some(ok.clone()),
        Err(_) => None,
    };

    let verify_result = VerifyResult {
        exec,
        stats: verify.stats,
    };

    if verify.args.batch {
        // serialize the result
        serde_json::to_writer(std::io::stdout(), &verify_result)?;
    }
    // get the actual exec result
    let exec_result = match exec_result {
        Ok(ok) => ok,
        Err(err) => return Err(err),
    };

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

struct Verify {
    args: super::Cli,
    verify_args: Cli,
    stats: VerifyStats,
}
