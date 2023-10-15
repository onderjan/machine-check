use log::info;
use machine_check_common::ExecResult;
use serde::{Deserialize, Serialize};

use crate::{CheckError, Cli, VerifyCli};

mod build;
mod execute;
mod translate;
mod work;

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyStats {
    pub transcription_time: Option<f64>,
    pub build_time: Option<f64>,
    pub execution_time: Option<f64>,
    pub prepared: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyResult {
    pub exec: Option<ExecResult>,
    pub stats: VerifyStats,
}

pub fn run(args: Cli, verify_args: VerifyCli) -> Result<(), CheckError> {
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
    args: Cli,
    verify_args: VerifyCli,
    stats: VerifyStats,
}
