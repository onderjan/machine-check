use machine_check_common::ExecResult;
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

use crate::{util::log_process_output, Error};

use super::Config;

pub(super) fn execute(config: &Config, artifact_path: &Path) -> Result<ExecResult, Error> {
    let mut command = Command::new(artifact_path);

    command.arg("--property");
    command.arg("AG![safe == 1]");

    // batch output so we can parse it
    command.arg("--batch");

    // forward property
    if let Some(property) = &config.property {
        command.arg("--property").arg(property);
    }

    // forward verbose
    for _ in 0..config.verbose {
        command.arg("--verbose");
    }

    // forward use-decay
    if config.use_decay {
        command.arg("--use-decay");
    }

    // the machine executable logs on stderr and gives us the result on stdout
    // pipe stdout and inherit stderr
    command.stdout(Stdio::piped()).stderr(Stdio::inherit());

    let exec_output = command.output().map_err(Error::ExecRun)?;

    if config.batch {
        // echo the batch output written to stdout
        std::io::stdout()
            .write_all(&exec_output.stdout)
            .map_err(Error::WriteStdout)?;
    }

    if !exec_output.status.success() {
        // stderr is already piped, do not print any error log
        return Err(Error::ExecStatus(exec_output.status));
    }

    log_process_output("Execution", &exec_output);

    let exec_result: ExecResult = serde_json::from_slice(&exec_output.stdout)?;

    Ok(exec_result)
}
