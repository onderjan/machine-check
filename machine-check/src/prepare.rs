use anyhow::anyhow;
use cargo_metadata::{camino::Utf8PathBuf, Message};
use log::info;
use std::{collections::BTreeSet, io::Write};
use std::{
    fs::File,
    io::BufWriter,
    process::{Command, Stdio},
};

#[derive(Debug)]
struct Rdep {
    target_name: String,
    paths: Vec<Utf8PathBuf>,
}

use serde::{Deserialize, Serialize};

use crate::{Cli, PrepareCli};

#[derive(Debug, Serialize, Deserialize)]
pub struct Preparation {
    pub target_build_args: Vec<String>,
}

pub(super) fn default_preparation_dir() -> Result<Utf8PathBuf, anyhow::Error> {
    // directory 'preparation' under the executable
    let mut path = std::env::current_exe()?;
    path.pop();
    let path = Utf8PathBuf::try_from(path)?;
    Ok(path.join("preparation"))
}

pub(super) fn prepare(_: Cli, prepare: PrepareCli) -> Result<(), anyhow::Error> {
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
    std::fs::create_dir_all(&home_dir)?;
    let target_dir = preparation_dir.join("target");
    std::fs::create_dir_all(&target_dir)?;
    let profile = String::from("release");

    // cargo build machine_check_exec and copy the dependencies to a separate directory
    let mut command = Command::new("cargo")
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
        .stderr(Stdio::null())
        .env("CARGO_HOME", &home_dir)
        .spawn()?;

    let output = command.wait()?;
    if !output.success() {
        return Err(anyhow!("Build was not successful"));
    }
    let reader = std::io::BufReader::new(
        command
            .stdout
            .take()
            .ok_or_else(|| anyhow!("Could not take build stdout"))?,
    );

    let mut linked_paths = BTreeSet::new();

    // get a list of paths to rlibs
    let mut rlibs = Vec::<Rdep>::new();
    for message in cargo_metadata::Message::parse_stream(reader) {
        let message = message?;
        let artifact = match message {
            Message::BuildScriptExecuted(build_script) => {
                // add linked paths
                linked_paths.extend(build_script.linked_paths);
                continue;
            }
            Message::CompilerArtifact(artifact) => artifact,
            Message::CompilerMessage(_) => {
                // we do not care
                continue;
            }
            Message::BuildFinished(finished) => {
                // should never have successful exit status if build was unsuccessful
                assert!(finished.success);
                continue;
            }
            _ => {
                return Err(anyhow!("Unknown cargo message: {:?}", message));
            }
        };

        for file_path in &artifact.filenames {
            let Some(extension) = file_path.extension() else {
                continue;
            };
            if extension == "rlib" {
                rlibs.push(Rdep {
                    target_name: artifact.target.name.clone(),
                    paths: artifact.filenames.clone(),
                });
            }
        }
    }

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

    // add extern
    for rlib in rlibs {
        // copy path-specified to exec build dir
        for original_path in rlib.paths {
            // TODO: base addition of extern on Target_Cargo.toml
            if matches!(rlib.target_name.as_str(), "mck" | "machine-check-exec") {
                // add extern to args
                // replace hyphens with underscores for rustc
                let extern_target_name = rlib.target_name.replace('-', "_");
                target_build_args.push(String::from("--extern"));
                target_build_args.push(format!("{}={}", extern_target_name, original_path));
            }
        }
    }

    // add linked paths
    for linked_path in linked_paths {
        target_build_args.push(String::from("-L"));
        target_build_args.push(linked_path.to_string());
    }

    let preparation = Preparation { target_build_args };

    let preparation_path = preparation_dir.join("preparation.json");
    let file = File::create(preparation_path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, &preparation)?;
    writer.flush()?;

    info!("Preparation complete.");
    Ok(())
}
