use cargo_metadata::{camino::Utf8PathBuf, Message};
use log::info;
use std::{collections::BTreeSet, io::Write};
use std::{
    fs::File,
    io::BufWriter,
    process::{Command, Stdio},
};

use serde::{Deserialize, Serialize};

use crate::util::log_process_output;
use crate::{CheckError, Cli, PrepareCli};

#[derive(Debug, Serialize, Deserialize)]
pub struct Preparation {
    pub target_build_args: Vec<String>,
}

pub(super) fn default_preparation_dir() -> Result<Utf8PathBuf, CheckError> {
    // directory 'preparation' under the executable
    let mut path = std::env::current_exe().map_err(CheckError::CurrentExe)?;
    path.pop();
    let path =
        Utf8PathBuf::try_from(path.clone()).map_err(|err| CheckError::PathToUtf8(path, err))?;
    Ok(path.join("preparation"))
}

pub(super) fn prepare(_: Cli, prepare: PrepareCli) -> Result<(), CheckError> {
    let preparation_dir = match prepare.preparation_path {
        Some(preparation_path) => preparation_path,
        None => {
            // use the default directory
            default_preparation_dir()?
        }
    };

    info!(
        "Preparing sub-artifacts for machine executable building into {:?}.",
        preparation_dir
    );

    let home_dir = preparation_dir.join("home");
    std::fs::create_dir_all(&home_dir)
        .map_err(|err| CheckError::CreateDir(home_dir.clone(), err))?;
    let target_dir = preparation_dir.join("target");
    std::fs::create_dir_all(&target_dir)
        .map_err(|err| CheckError::CreateDir(target_dir.clone(), err))?;
    let profile = String::from("release");

    // cargo build machine_check_exec and copy the dependencies to a separate directory
    let build_output = Command::new("cargo")
        .arg("build")
        .arg("--package")
        .arg("machine-check-exec")
        .arg("--lib")
        .arg("--profile")
        .arg(&profile)
        .arg("--message-format=json-render-diagnostics")
        .arg("--target-dir")
        .arg(&target_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("CARGO_HOME", &home_dir)
        .output()
        .map_err(CheckError::BuildRun)?;

    log_process_output(&build_output);

    if !build_output.status.success() {
        return Err(CheckError::BuildStatus(build_output.status));
    }

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
        let message = message.map_err(CheckError::CargoParse)?;
        match message {
            Message::BuildScriptExecuted(build_script) => {
                // add linked paths
                linked_paths.extend(build_script.linked_paths);
            }
            Message::CompilerArtifact(artifact) => {
                if matches!(artifact.target.name.as_str(), "mck" | "machine-check-exec") {
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
        .map_err(|err| CheckError::CreateFile(preparation_path.clone(), err))?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, &preparation)?;
    writer
        .flush()
        .map_err(|err| CheckError::Flush(preparation_path, err))?;

    info!("Preparation complete.");
    Ok(())
}
