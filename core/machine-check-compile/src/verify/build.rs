use camino::Utf8Path;
use cargo_metadata::camino::Utf8PathBuf;
use log::{debug, warn};
use std::{
    collections::HashMap,
    fs::{self},
    path::PathBuf,
    process::{Command, Stdio},
};

use crate::{
    prepare::{default_preparation_dir, Preparation},
    util::{log_process_error_log, log_process_output},
    Error,
};

use super::{Config, Stats};

pub(super) fn build(
    config: &Config,
    stats: &mut Stats,
    machine_package_dir_path: &Utf8Path,
) -> Result<PathBuf, Error> {
    fs::create_dir_all(machine_package_dir_path)
        .map_err(|err| Error::CreateDir(machine_package_dir_path.to_path_buf(), err))?;
    let machine_target_dir_path = machine_package_dir_path.join("build-target");
    fs::create_dir_all(&machine_target_dir_path)
        .map_err(|err| Error::CreateDir(machine_target_dir_path.clone(), err))?;

    // use the default preparation directory if it exists
    let preparation_path = match &config.preparation_path {
        Some(path) => Some(path.clone()),
        None => {
            let default_path = default_preparation_dir()?;
            if default_path.exists() {
                Some(default_path)
            } else {
                None
            }
        }
    };

    stats.prepared = Some(preparation_path.is_some());

    // use rustc if there is preparation, use cargo if there is no preparation
    let (is_rustc, mut build_command) = match preparation_path {
        Some(preparation_path) => {
            // read the preparation definition file
            let preparation_file_path = preparation_path.join("preparation.json");
            debug!(
                "Reading preparation definition from {:?}.",
                preparation_file_path
            );
            let preparation_string = std::fs::read_to_string(&preparation_file_path)
                .map_err(|err| Error::ReadFile(preparation_file_path, err))?;
            let preparation: Preparation = serde_json::from_str(preparation_string.as_str())?;

            // main is located in package/src/main.rs for compatibility with cargo
            let mut main_path = machine_package_dir_path.to_path_buf();
            main_path.push("src");
            main_path.push("main.rs");

            // compose the build command
            let mut build_command = Command::new("rustc");
            build_command
                .arg(main_path)
                .arg("--out-dir")
                .arg(machine_target_dir_path)
                .args(preparation.target_build_args);
            (true, build_command)
        }
        None => {
            warn!("Prepared artifacts not found, use the prepare subcommand to speed up builds");
            // add package Cargo.toml and build as normal Cargo release binary
            let machine_package_cargo_toml = include_str!("../../resources/Target_Cargo.toml");
            let machine_package_cargo_toml_path = machine_package_dir_path.join("Cargo.toml");
            debug!(
                "Writing machine package Cargo.toml to {:?}.",
                machine_package_cargo_toml_path
            );
            fs::write(&machine_package_cargo_toml_path, machine_package_cargo_toml)
                .map_err(|err| Error::WriteFile(machine_package_cargo_toml_path.clone(), err))?;

            // compose the build command
            let mut build_command = Command::new("cargo");
            build_command
                .arg("build")
                .arg("--message-format=json-diagnostic-short")
                .arg("--manifest-path")
                .arg(machine_package_cargo_toml_path)
                .arg("--target-dir")
                .arg(machine_target_dir_path)
                .arg("--bin")
                .arg("machine-check-exec-target")
                .arg("--release");
            (false, build_command)
        }
    };
    debug!("Executing build command {:?}.", build_command);

    // build with piped
    let build_output = build_command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(Error::BuildRun)?;

    debug!("Build command status: {:?}.", build_command.status());

    if !build_output.status.success() {
        log_process_error_log(
            "Build",
            if is_rustc {
                &build_output.stdout
            } else {
                &build_output.stderr
            },
        );
        return Err(Error::BuildStatus(build_output.status));
    }
    log_process_output("Build", &build_output);

    debug!("Determining executable path.");

    let mut executable_path: Option<Utf8PathBuf> = None;
    let mut build_amount = 0;
    // parse output
    if is_rustc {
        // simple lines of JSON, find a line that contains the artifact
        // rustc prints the messages to stderr
        let stderr = String::from_utf8(build_output.stderr).map_err(Error::RustcParse)?;
        for line in stderr.lines() {
            let hash_map: HashMap<String, String> = serde_json::from_str(line)?;
            if let (Some(artifact), Some(emit)) = (hash_map.get("artifact"), hash_map.get("emit")) {
                if emit == "link" {
                    // this is the executable
                    build_amount += 1;
                    executable_path.get_or_insert(artifact.into());
                }
            }
        }
    } else {
        // parse with the cargo metadata crate
        // cargo prints the messages to stdout
        let bytes: &[u8] = &build_output.stdout;
        for message in cargo_metadata::Message::parse_stream(bytes) {
            let message = message.map_err(Error::CargoParse)?;
            if let cargo_metadata::Message::CompilerArtifact(artifact) = message {
                if let Some(artifact_executable_path) = artifact.executable {
                    // this is the executable
                    build_amount += 1;
                    executable_path.get_or_insert(artifact_executable_path);
                }
            }
        }
    }
    if build_amount != 1 {
        return Err(Error::BuildAmount(build_amount));
    }
    let Some(executable_path) = executable_path else {
        return Err(Error::BuildAmount(build_amount));
    };
    debug!("Built machine-verifier executable {:?}.", executable_path);
    Ok(PathBuf::from(executable_path))
}
