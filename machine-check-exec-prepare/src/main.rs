use cargo_metadata::{
    camino::{Utf8Path, Utf8PathBuf},
    Message,
};
use machine_check_exec_prepare::Preparation;

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

fn main() {
    let exec_build_dir = Utf8Path::new("./resources/exec-build");
    let home_dir = exec_build_dir.join("home");
    std::fs::create_dir_all(&home_dir).expect("Exec build home dir should be created");
    let target_dir = exec_build_dir.join("target");
    std::fs::create_dir_all(&target_dir).expect("Exec build target dir should be created");
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
        .spawn()
        .unwrap();

    let output = command.wait().expect("Couldn't get cargo's exit status");
    if !output.success() {
        eprintln!("Cargo build was not successful");
        std::process::exit(-1);
    }
    let reader = std::io::BufReader::new(command.stdout.take().unwrap());

    let mut linked_paths = BTreeSet::new();

    // get a list of paths to rlibs
    let mut rlibs = Vec::<Rdep>::new();
    for message in cargo_metadata::Message::parse_stream(reader) {
        let unwrapped = message.unwrap();
        println!("Message: {:?}", unwrapped);
        let artifact = match unwrapped {
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
                panic!("Unknown cargo message: {:?}", unwrapped);
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

    // create directory for the resources
    let exec_build_dir = Utf8Path::new("./resources/exec-build");

    let mut target_build_args = Vec::<String>::new();

    // add linked dependency which is in target
    target_build_args.push(String::from("-L"));
    target_build_args.push(format!("dependency={}/{}/deps", target_dir, profile));

    // add extern
    for rlib in rlibs {
        // copy path-specified to exec build dir
        for original_path in rlib.paths {
            // TODO: base addition of extern on Cargo.toml
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
        println!("Linked path: {}", linked_path);
        target_build_args.push(String::from("-L"));
        target_build_args.push(linked_path.to_string());
    }

    /*for rlib in rlibs {
        // copy path-specified to exec build dir
        for original_path in rlib.paths {
            let rlib_filename = original_path
                .file_name()
                .expect("Rlib path should have filename");
            let exec_build_rlib_path = lib_dir.join(rlib_filename);

            std::fs::copy(original_path, exec_build_rlib_path)
                .expect("Rlib should be copyable to exec build dir");
        }

        // TODO: base addition of extern on Cargo.toml
        if matches!(rlib.target_name.as_str(), "mck" | "machine-check-exec") {
            // add extern to args
            // replace hyphens with underscores for rustc
            let extern_target_name = rlib.target_name.replace('-', "_");
            target_build_args.push(String::from("--extern"));
            target_build_args.push(format!("{}={}", extern_target_name, rlib.rlib_path));
        }
    }

    let deps_dir = exec_build_dir.join("deps");
    std::fs::create_dir_all(deps_dir.clone()).expect("Deps dir should be created");

    // copy link dependencies to deps
    for linked_path in linked_paths {
        println!("Linked path: {}", linked_path);
        // we are only concerned about those with the 'native=' prefix, strip it
        let linked_path_full_string = linked_path.to_string();
        let Some(linked_path) = linked_path_full_string.strip_prefix("native=") else {
            continue;
        };

        let paths_inside = std::fs::read_dir(linked_path).unwrap();

        for path_inside in paths_inside {
            let entry = path_inside.expect("Should be able to look at path inside");
            if entry.file_type().unwrap().is_file() {
                // copy to deps directory
                let dep_path = entry.path();
                println!("Path inside: {:?}", entry.path());

                let exec_build_deps_path =
                    deps_dir.join(Utf8PathBuf::from_path_buf(entry.file_name().into()).unwrap());
                println!("Copying to path: {:?}", exec_build_deps_path);
                let mut source = std::fs::File::open(&dep_path).unwrap();
                let mut target = std::fs::File::create(&exec_build_deps_path).unwrap();
                std::io::copy(&mut source, &mut target)
                    .expect("Dependency should be copyable to exec deps dir");
            }
        }

        //target_build_args.push(String::from("-L"));
        //target_build_args.push(format!("dependency={}", linked_path));
    }

    // add link dependency to args
    target_build_args.push(String::from("-L"));
    //target_build_args.push(format!("dependency={}", deps_dir));
    target_build_args.push(String::from(
        "dependency=C:\\Users\\Mallory\\rust\\machine-check\\target\\debug\\deps",
    ));*/

    // add link native to args
    //target_build_args.push(String::from("-L"));
    //target_build_args.push(format!("native={}", lib_dir));

    let preparation = Preparation { target_build_args };

    let preparation_path = exec_build_dir.join("preparation.json");
    let file = File::create(preparation_path).unwrap();
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, &preparation).unwrap();
    writer.flush().unwrap();
}
