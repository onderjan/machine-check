use machine_check_common::ExecResult;
use std::{
    path::Path,
    process::{Command, Stdio},
};

use crate::{util::log_process_output, CheckError};

use super::Verify;

impl Verify {
    pub(super) fn execute(&self, artifact_path: &Path) -> Result<ExecResult, CheckError> {
        let mut command = Command::new(artifact_path);

        // forward batch
        if self.args.batch {
            command.arg("--batch");
        }
        // forward property
        if let Some(property) = &self.verify_args.property {
            command.arg("--property").arg(property);
        }

        // forward verbose
        for _ in 0..self.args.verbose {
            command.arg("--verbose");
        }

        // forward use-decay
        if self.verify_args.use_decay {
            command.arg("--use-decay");
        }

        // the machine executable logs on stderr and gives us the result on stdout
        // pipe stdout and inherit stderr
        command.stdout(Stdio::piped()).stderr(Stdio::inherit());

        let exec_output = command.output().map_err(CheckError::ExecRun)?;

        if !exec_output.status.success() {
            // stderr is already piped, do not print any error log
            return Err(CheckError::ExecStatus(exec_output.status));
        }

        log_process_output("Execution", &exec_output);

        let exec_result: ExecResult = serde_json::from_slice(&exec_output.stdout)?;

        Ok(exec_result)
    }
}
