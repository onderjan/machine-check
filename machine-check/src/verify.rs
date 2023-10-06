use anyhow::anyhow;
use camino::Utf8Path;
use cargo_metadata::camino::Utf8PathBuf;
use log::{debug, info, warn};
use machine_check_common::ExecResult;
use machine_check_lib::{create_abstract_machine, write_machine};
use std::{
    collections::HashMap,
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use syn::{parse_quote, Item, ItemFn};
use tempdir::TempDir;

use crate::{
    prepare::{self, Preparation},
    Cli, VerifyCli,
};

pub(super) fn run(args: Cli, verify_args: VerifyCli) -> Result<(), anyhow::Error> {
    Verify { args, verify_args }.work()
}

struct Verify {
    args: Cli,
    verify_args: VerifyCli,
}

impl Verify {
    fn work(&self) -> Result<(), anyhow::Error> {
        let system_path = Path::new(&self.verify_args.system_path);

        info!("Transcribing the system into a machine.");
        let cwd = std::env::current_dir()?;
        debug!("Current working directory is {:?}.", cwd);
        debug!("The system path is {:?}.", system_path);

        let btor2_file = match File::open(system_path) {
            Ok(file) => file,
            Err(err) => return Err(anyhow!("Cannot open input file {:?}: {}", system_path, err)),
        };

        let concrete_machine: syn::File = syn::parse2(btor2rs::translate_file(btor2_file)?)?;
        let mut abstract_machine = create_abstract_machine(&concrete_machine)?;

        // add main function

        let main_fn: ItemFn = parse_quote!(
            fn main() {
                ::machine_check_exec::run::<mark::Machine>()
            }
        );
        abstract_machine.items.push(Item::Fn(main_fn));

        // the machine package directory path can be given
        // we will write the machine into a temporary directory if it is not given
        // do not drop temporary directory too early
        let (machine_package_dir_path, machine_package_temp_dir) =
            match &self.verify_args.machine_path {
                Some(path) => (path.clone(), None),
                None => {
                    let temp_dir = TempDir::new("machine_check_machine_")?;
                    let temp_dir_path = Utf8PathBuf::try_from(temp_dir.path().to_path_buf())?;
                    (temp_dir_path, Some(temp_dir))
                }
            };

        let src_dir_path = machine_package_dir_path.join("src");
        fs::create_dir_all(&src_dir_path)?;
        let main_path = src_dir_path.join("main.rs");

        debug!("Writing the machine into file {:?}.", main_path);
        write_machine("abstract", &abstract_machine, main_path.as_path())?;

        info!("Building a machine verifier.");
        let artifact_path = self.build_machine(&machine_package_dir_path)?;

        info!("Executing the machine verifier.");

        self.execute_machine(&artifact_path)?;

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

        Ok(())
    }

    fn build_machine(&self, machine_package_dir_path: &Utf8Path) -> Result<PathBuf, anyhow::Error> {
        fs::create_dir_all(machine_package_dir_path)?;
        let machine_target_dir_path = machine_package_dir_path.join("build-target");
        fs::create_dir_all(&machine_target_dir_path)?;

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

        // use rustc if there is preparation, use cargo if there is no preparation
        let (is_rustc, mut build_command) = match preparation_path {
            Some(preparation_path) => {
                // read the preparation definition file
                let preparation_file_path = preparation_path.join("preparation.json");
                debug!(
                    "Reading preparation definition from {:?}.",
                    preparation_file_path
                );
                let preparation_string = std::fs::read_to_string(preparation_file_path)?;
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
                fs::write(&machine_package_cargo_toml_path, machine_package_cargo_toml)?;

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
            .output()?;

        debug!("Build command status: {:?}.", build_command.status());

        if !build_output.status.success() {
            // TODO: get the errors from JSON
            let human_output = if is_rustc {
                // rustc prints human-readable to stdout
                build_output.stdout
            } else {
                // cargo prints human-readable to stderr
                build_output.stderr
            };
            return Err(anyhow!(
                "Build was not successful:\n{}",
                String::from_utf8(human_output)?
            ));
        }

        debug!("Determining executable path.");

        let mut executable_path: Option<Utf8PathBuf> = None;
        // parse output
        if is_rustc {
            // simple lines of JSON, find a line that contains the artifact
            // rustc prints the messages to stderr
            let stderr = String::from_utf8(build_output.stderr)?;
            for line in stderr.lines() {
                let hash_map: HashMap<String, String> = serde_json::from_str(line)?;
                if let (Some(artifact), Some(emit)) =
                    (hash_map.get("artifact"), hash_map.get("emit"))
                {
                    if emit == "link" {
                        // this is the executable
                        if executable_path.is_some() {
                            return Err(anyhow!("Multiple executables were built"));
                        }
                        executable_path = Some(artifact.into());
                    }
                }
            }
        } else {
            // parse with the cargo metadata crate
            // cargo prints the messages to stdout
            let bytes: &[u8] = &build_output.stdout;
            for message in cargo_metadata::Message::parse_stream(bytes) {
                let message = message?;
                if let cargo_metadata::Message::CompilerArtifact(artifact) = message {
                    if let Some(artifact_executable_path) = artifact.executable {
                        if executable_path.is_some() {
                            return Err(anyhow!("Multiple executables were built"));
                        }
                        executable_path = Some(artifact_executable_path);
                    }
                }
            }
        }
        let Some(executable_path) = executable_path else {
            return Err(anyhow!("Build generated no executable"));
        };
        debug!("Built machine-verifier executable {:?}.", executable_path);
        Ok(PathBuf::from(executable_path))
    }

    fn execute_machine(&self, artifact_path: &Path) -> Result<(), anyhow::Error> {
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

        // the machine executable logs on stderr and gives us the result on stdout
        // pipe stdout and inherit stderr
        command.stdout(Stdio::piped()).stderr(Stdio::inherit());

        let exec_output = command.output().unwrap();

        if !exec_output.status.success() {
            return Err(anyhow!("Execution was not successful"));
        }

        let exec_result: ExecResult = serde_json::from_slice(&exec_output.stdout)?;
        if self.args.batch {
            // serialize the result after determining it is JSON-parsable by deserialization
            serde_json::to_writer(std::io::stdout(), &exec_result)?;
        } else {
            // print interesting facts
            info!(
                "Used {} states and {} refinements.",
                exec_result.info.num_states, exec_result.info.num_refinements
            );
            match exec_result.conclusion {
                Ok(conclusion) => {
                    info!("Conclusion: {}", conclusion);
                }
                Err(err) => return Err(err.into()),
            }
        }

        Ok(())
    }
}
