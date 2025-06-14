use cargo_metadata::{camino::Utf8PathBuf, Message};
use log::info;
use std::{collections::BTreeSet, io::Write};
use std::{
    fs::File,
    io::BufWriter,
    process::{Command, Stdio},
};

use serde::{Deserialize, Serialize};

use crate::features::add_cargo_features;
use crate::util::{log_process_error_log, log_process_output};
use crate::Error;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Preparation {
    pub target_build_args: Vec<String>,
}

pub fn default_preparation_dir() -> Result<Utf8PathBuf, Error> {
    // directory 'machine-check-preparation' under the executable
    let mut path = std::env::current_exe().map_err(Error::CurrentExe)?;
    path.pop();
    let path = Utf8PathBuf::try_from(path.clone()).map_err(|err| Error::PathToUtf8(path, err))?;
    Ok(path.join("machine-check-preparation"))
}

pub struct Config {
    pub preparation_path: Option<Utf8PathBuf>,
    pub clean: bool,
}

pub fn prepare(config: Config) -> Result<(), Error> {
    let preparation_dir = match config.preparation_path {
        Some(preparation_path) => preparation_path,
        None => {
            // use the default directory
            default_preparation_dir()?
        }
    };

    if config.clean {
        info!(
            "Cleaning preparation by removing directory {:?}.",
            preparation_dir
        );
        std::fs::remove_dir_all(preparation_dir.clone())
            .map_err(|err| Error::RemoveDirAll(preparation_dir, err))?;
        return Ok(());
    }

    info!(
        "Preparing sub-artifacts for machine executable building into {:?}.",
        preparation_dir
    );

    let src_dir_path = preparation_dir.join("src");
    std::fs::create_dir_all(&src_dir_path)
        .map_err(|err| Error::CreateDir(src_dir_path.clone(), err))?;
    let lib_path = src_dir_path.join("lib.rs");

    std::fs::write(lib_path.clone(), "").map_err(|err| Error::WriteFile(lib_path, err))?;

    let cargo_toml = include_str!("../resources/Prepare_Cargo.toml");
    let cargo_toml_path = preparation_dir.join("Cargo.toml");
    std::fs::write(&cargo_toml_path, cargo_toml)
        .map_err(|err| Error::WriteFile(cargo_toml_path.clone(), err))?;

    let home_dir = preparation_dir.join("home");
    std::fs::create_dir_all(&home_dir).map_err(|err| Error::CreateDir(home_dir.clone(), err))?;
    let target_dir = preparation_dir.join("target");
    std::fs::create_dir_all(&target_dir)
        .map_err(|err| Error::CreateDir(target_dir.clone(), err))?;
    let profile = String::from("release");

    // cargo build machine_check_exec and copy the dependencies to a separate directory
    let mut build_command = Command::new("cargo");
    build_command
        .arg("build")
        .arg("--manifest-path")
        .arg(cargo_toml_path)
        .arg("--lib")
        .arg("--profile")
        .arg(&profile)
        .arg("--message-format=json-render-diagnostics")
        .arg("--target-dir")
        .arg(&target_dir);

    add_cargo_features(&mut build_command);

    let build_output = build_command
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .env("CARGO_HOME", &home_dir)
        .output()
        .map_err(Error::BuildRun)?;

    if !build_output.status.success() {
        log_process_error_log("Preparation", &build_output.stderr);
        return Err(Error::BuildStatus(build_output.status));
    }

    log_process_output("Preparation", &build_output);

    let mut linked_paths = BTreeSet::new();

    let mut target_build_args = vec![
        String::from("--edition=2021"),
        String::from("--error-format=json"),
        String::from("--json=artifacts"),
        String::from("--crate-type"),
        String::from("bin"),
        String::from("-C"),
        String::from("opt-level=3"),
        String::from("-C"),
        String::from("embed-bitcode=no"),
        String::from("-C"),
        String::from("strip=symbols"),
    ];

    // add linked dependency which is in target
    let deps_dir = target_dir.join(profile).join("deps");

    target_build_args.push(String::from("-L"));
    target_build_args.push(format!("dependency={}", deps_dir));

    // get a list of paths to rlibs
    let bytes: &[u8] = &build_output.stdout;
    for message in cargo_metadata::Message::parse_stream(bytes) {
        let message = message.map_err(Error::CargoParse)?;
        match message {
            Message::BuildScriptExecuted(build_script) => {
                // add linked paths
                linked_paths.extend(build_script.linked_paths);
            }
            Message::CompilerArtifact(artifact) => {
                // replace target name hyphens with underscores
                // this will also be needed for rustc
                let target_name = artifact.target.name.replace('-', "_");
                if matches!(
                    target_name.as_str(),
                    "mck" | "machine_check" | "machine_check_exec"
                ) {
                    for original_path in artifact.filenames {
                        // TODO: base addition of extern on Target_Cargo.toml
                        // add extern to args
                        // replace hyphens with underscores for rustc
                        let extern_target_name = artifact.target.name.replace('-', "_");
                        target_build_args.push(String::from("--extern"));
                        target_build_args.push(format!("{}={}", extern_target_name, original_path));
                    }
                }
            }
            Message::BuildFinished(finished) => {
                // should never have successful exit status if build was unsuccessful
                assert!(finished.success);
            }
            _ => (),
        };
    }

    // add linked paths
    for linked_path in linked_paths {
        target_build_args.push(String::from("-L"));
        target_build_args.push(linked_path.to_string());
    }

    let preparation = Preparation { target_build_args };

    let preparation_path = preparation_dir.join("preparation.json");
    let file = File::create(&preparation_path)
        .map_err(|err| Error::CreateFile(preparation_path.clone(), err))?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, &preparation)?;
    writer
        .flush()
        .map_err(|err| Error::Flush(preparation_path, err))?;

    info!("Preparation complete.");
    Ok(())
}
