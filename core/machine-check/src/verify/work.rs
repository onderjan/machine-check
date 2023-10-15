use log::{debug, info, warn};
use machine_check_common::ExecResult;
use std::time::Instant;

use crate::CheckError;

use super::Verify;

impl Verify {
    pub(super) fn work(&mut self) -> Result<ExecResult, CheckError> {
        let system_path = &self.verify_args.system_path;

        info!("Transcribing the system into a machine.");
        let cwd = std::env::current_dir().map_err(CheckError::WorkDir)?;
        debug!("Current working directory is {:?}.", cwd);
        debug!("The system path is {:?}.", system_path);

        let transcription_start = Instant::now();
        let (machine_package_dir_path, machine_package_temp_dir) = self.translate()?;
        self.stats.transcription_time = Some(transcription_start.elapsed().as_secs_f64());

        info!("Building a machine verifier.");
        let build_start = Instant::now();
        let artifact_path = self.build(&machine_package_dir_path)?;
        self.stats.build_time = Some(build_start.elapsed().as_secs_f64());

        info!("Executing the machine verifier.");

        let execution_start = Instant::now();
        let exec_result = self.execute(&artifact_path)?;
        self.stats.execution_time = Some(execution_start.elapsed().as_secs_f64());

        // warn on error to close the temporary directory, it is not critical
        if let Some(temp_dir) = machine_package_temp_dir {
            debug!("Deleting temporary directory {:?}", temp_dir.path());
            if let Err(err) = temp_dir.close() {
                warn!(
                    "Could not close temporary directory for machine: {:#?}",
                    err
                );
            }
        }

        Ok(exec_result)
    }
}
