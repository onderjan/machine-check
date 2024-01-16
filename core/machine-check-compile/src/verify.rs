mod build;
mod execute;

use std::time::Instant;

use camino::Utf8PathBuf;
use log::{debug, info, warn};
use machine_check_common::ExecResult;
use serde::{Deserialize, Serialize};
use tempdir::TempDir;

use crate::Error;

pub struct Config {
    pub abstract_machine: syn::File,
    pub machine_path: Option<Utf8PathBuf>,
    pub preparation_path: Option<Utf8PathBuf>,
    pub batch: bool,
    pub property: Option<String>,
    pub verbose: u8,
    pub use_decay: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Stats {
    pub transcription_time: Option<f64>,
    pub build_time: Option<f64>,
    pub execution_time: Option<f64>,
    pub prepared: Option<bool>,
}

pub fn verify(config: Config) -> Result<ExecResult, Error> {
    let mut stats = Stats {
        transcription_time: None,
        build_time: None,
        execution_time: None,
        prepared: None,
    };

    info!("Transcribing the system into a machine.");
    let map_err = std::env::current_dir().map_err(Error::WorkDir);
    let cwd = map_err?;
    debug!("Current working directory is {:?}.", cwd);

    let transcription_start = Instant::now();
    let (machine_package_dir_path, machine_package_temp_dir) = write_machine(&config)?;
    stats.transcription_time = Some(transcription_start.elapsed().as_secs_f64());

    info!("Building a machine verifier.");
    let build_start = Instant::now();
    let artifact_path = build::build(&config, &mut stats, &machine_package_dir_path)?;
    stats.build_time = Some(build_start.elapsed().as_secs_f64());

    info!("Executing the machine verifier.");

    let execution_start = Instant::now();
    let exec_result = execute::execute(&config, &artifact_path)?;
    stats.execution_time = Some(execution_start.elapsed().as_secs_f64());

    info!("Stats: {:?}", stats);

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

fn write_machine(arguments: &Config) -> Result<(Utf8PathBuf, Option<TempDir>), Error> {
    // the machine package directory path can be given
    // we will write the machine into a temporary directory if it is not given
    // do not drop temporary directory too early
    let (machine_package_dir_path, machine_package_temp_dir) = match &arguments.machine_path {
        Some(path) => (path.clone(), None),
        None => {
            let temp_dir = TempDir::new("machine_check_machine_").map_err(Error::CreateTempDir)?;
            let temp_dir_path = temp_dir.path().to_path_buf();
            let temp_dir_path = Utf8PathBuf::try_from(temp_dir_path.clone())
                .map_err(|err| Error::PathToUtf8(temp_dir_path, err))?;
            (temp_dir_path, Some(temp_dir))
        }
    };

    let src_dir_path = machine_package_dir_path.join("src");
    std::fs::create_dir_all(&src_dir_path)
        .map_err(|err| Error::CreateDir(src_dir_path.clone(), err))?;
    let main_path = src_dir_path.join("main.rs");

    machine_check_machine::write_machine(&arguments.abstract_machine, &main_path)?;
    Ok((machine_package_dir_path, machine_package_temp_dir))
}
