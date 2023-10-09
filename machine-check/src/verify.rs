use camino::Utf8Path;
use cargo_metadata::camino::Utf8PathBuf;
use log::{debug, info, warn};
use machine_check_common::ExecResult;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Instant,
};
use syn::{parse_quote, Item, ItemFn};
use tempdir::TempDir;

use crate::{
    machine::{create_abstract_machine, write_machine},
    prepare::{self, Preparation},
    util::log_process_output,
    CheckError, Cli, VerifyCli,
};

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

impl Verify {
    fn work(&mut self) -> Result<ExecResult, CheckError> {
        let system_path = &self.verify_args.system_path;

        info!("Transcribing the system into a machine.");
        let cwd = std::env::current_dir().map_err(CheckError::WorkDir)?;
        debug!("Current working directory is {:?}.", cwd);
        debug!("The system path is {:?}.", system_path);

        let transcription_start = Instant::now();
        let (machine_package_dir_path, machine_package_temp_dir) = self.transcribe_machine()?;
        self.stats.transcription_time = Some(transcription_start.elapsed().as_secs_f64());

        info!("Building a machine verifier.");
        let build_start = Instant::now();
        let artifact_path = self.build_machine(&machine_package_dir_path)?;
        self.stats.build_time = Some(build_start.elapsed().as_secs_f64());

        info!("Executing the machine verifier.");

        let execution_start = Instant::now();
        let exec_result = self.execute_machine(&artifact_path)?;
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

    fn transcribe_machine(&self) -> Result<(Utf8PathBuf, Option<TempDir>), CheckError> {
        let btor2_file = File::open(&self.verify_args.system_path)
            .map_err(|err| CheckError::OpenFile(self.verify_args.system_path.clone(), err))?;

        // the machine package directory path can be given
        // we will write the machine into a temporary directory if it is not given
        // do not drop temporary directory too early
        let (machine_package_dir_path, machine_package_temp_dir) =
            match &self.verify_args.machine_path {
                Some(path) => (path.clone(), None),
                None => {
                    let temp_dir = TempDir::new("machine_check_machine_")
                        .map_err(CheckError::CreateTempDir)?;
                    let temp_dir_path = temp_dir.path().to_path_buf();
                    let temp_dir_path = Utf8PathBuf::try_from(temp_dir_path.clone())
                        .map_err(|err| CheckError::PathToUtf8(temp_dir_path, err))?;
                    (temp_dir_path, Some(temp_dir))
                }
            };

        let src_dir_path = machine_package_dir_path.join("src");
        fs::create_dir_all(&src_dir_path)
            .map_err(|err| CheckError::CreateDir(src_dir_path.clone(), err))?;
        let main_path = src_dir_path.join("main.rs");

        let translation = machine_check_transcribe_btor2::translate_file(btor2_file)
            .map_err(CheckError::TranslateFromBtor2)?;
        let concrete_machine: syn::File =
            syn::parse2(translation).map_err(CheckError::SyntaxTree)?;
        let mut abstract_machine =
            create_abstract_machine(&concrete_machine).map_err(CheckError::AbstractMachine)?;

        // add main function

        let main_fn: ItemFn = parse_quote!(
            fn main() {
                ::machine_check_exec::run::<mark::Machine>()
            }
        );
        abstract_machine.items.push(Item::Fn(main_fn));

        debug!("Writing the machine into file {:?}.", main_path);
        write_machine(&abstract_machine, &main_path)?;
        Ok((machine_package_dir_path, machine_package_temp_dir))
    }

    fn build_machine(
        &mut self,
        machine_package_dir_path: &Utf8Path,
    ) -> Result<PathBuf, CheckError> {
        fs::create_dir_all(machine_package_dir_path)
            .map_err(|err| CheckError::CreateDir(machine_package_dir_path.to_path_buf(), err))?;
        let machine_target_dir_path = machine_package_dir_path.join("build-target");
        fs::create_dir_all(&machine_target_dir_path)
            .map_err(|err| CheckError::CreateDir(machine_target_dir_path.clone(), err))?;

        // use the default preparation directory if it exists
        let preparation_path = match &self.verify_args.preparation_path {
            Some(path) => Some(path.clone()),
            None => {
                let default_path = prepare::default_preparation_dir()?;
                if default_path.exists() {
                    Some(default_path)
                } else {
                    None
                }
            }
        };

        self.stats.prepared = Some(preparation_path.is_some());

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
                    .map_err(|err| CheckError::ReadFile(preparation_file_path, err))?;
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
                warn!(
                    "Prepared artifacts not found, use the prepare subcommand to speed up builds"
                );
                // add package Cargo.toml and build as normal Cargo release binary
                let machine_package_cargo_toml = include_str!("../resources/Target_Cargo.toml");
                let machine_package_cargo_toml_path = machine_package_dir_path.join("Cargo.toml");
                debug!(
                    "Writing machine package Cargo.toml to {:?}.",
                    machine_package_cargo_toml_path
                );
                fs::write(&machine_package_cargo_toml_path, machine_package_cargo_toml).map_err(
                    |err| CheckError::WriteFile(machine_package_cargo_toml_path.clone(), err),
                )?;

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
            .map_err(CheckError::BuildRun)?;

        debug!("Build command status: {:?}.", build_command.status());

        log_process_output(&build_output);

        if !build_output.status.success() {
            return Err(CheckError::BuildStatus(build_output.status));
        }

        debug!("Determining executable path.");

        let mut executable_path: Option<Utf8PathBuf> = None;
        let mut build_amount = 0;
        // parse output
        if is_rustc {
            // simple lines of JSON, find a line that contains the artifact
            // rustc prints the messages to stderr
            let stderr = String::from_utf8(build_output.stderr).map_err(CheckError::RustcParse)?;
            for line in stderr.lines() {
                let hash_map: HashMap<String, String> = serde_json::from_str(line)?;
                if let (Some(artifact), Some(emit)) =
                    (hash_map.get("artifact"), hash_map.get("emit"))
                {
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
                let message = message.map_err(CheckError::CargoParse)?;
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
            return Err(CheckError::BuildAmount(build_amount));
        }
        let Some(executable_path) = executable_path else {
            return Err(CheckError::BuildAmount(build_amount));
        };
        debug!("Built machine-verifier executable {:?}.", executable_path);
        Ok(PathBuf::from(executable_path))
    }

    fn execute_machine(&self, artifact_path: &Path) -> Result<ExecResult, CheckError> {
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

        log_process_output(&exec_output);

        if !exec_output.status.success() {
            return Err(CheckError::ExecStatus(exec_output.status));
        }

        let exec_result: ExecResult = serde_json::from_slice(&exec_output.stdout)?;

        Ok(exec_result)
    }
}
