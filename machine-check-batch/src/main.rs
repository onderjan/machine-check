use anyhow::anyhow;
use std::{
    env,
    ffi::OsStr,
    fs, io,
    io::{Read, Stderr, Write},
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
use wait_timeout::ChildExt;
use walkdir::WalkDir;
use yaml_rust::YamlLoader;

fn check(path: &Path) -> anyhow::Result<Option<bool>> {
    let machine_check_toml = "./machine-check/Cargo.toml";
    let machine_check_exec_toml = "./machine-check-exec/Cargo.toml";
    let machine_check_output = Command::new("cargo")
        .arg("run")
        .arg("--manifest-path")
        .arg(machine_check_toml)
        .arg("--")
        .arg(path)
        .arg("--release")
        .output()?;

    if !machine_check_output.status.success() {
        return Err(anyhow!(
            "Non-success on machine-check, exit code {:?}",
            machine_check_output.status.code()
        ));
    }

    print!("[compiled] ");
    io::stdout().flush()?;

    let mut child = Command::new("cargo")
        .arg("run")
        .arg("--manifest-path")
        .arg(machine_check_exec_toml)
        .arg("--release")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();

    let secs = Duration::from_secs(10);
    let status = match child.wait_timeout(secs).unwrap() {
        Some(status) => status,
        None => {
            child.kill().unwrap();
            return Ok(None);
        }
    };

    print!("[executed] ");
    if !status.success() {
        return Err(anyhow!(
            "Non-success on machine-check-exec, exit code {:?}",
            status.code()
        ));
    }
    let mut exec_stdout = String::new();
    child
        .stdout
        .ok_or_else(|| anyhow!("Stdout is not OK"))?
        .read_to_string(&mut exec_stdout)
        .unwrap();
    match exec_stdout.as_str() {
        "Safe: true\n" => Ok(Some(true)),
        "Safe: false\n" => Ok(Some(false)),
        _ => Err(anyhow!("Unexpected stdout")),
    }
}

fn run(dir: &Path) -> anyhow::Result<()> {
    let mut num_correct_true: usize = 0;
    let mut num_correct_false: usize = 0;
    let mut num_wrong_true: usize = 0;
    let mut num_wrong_false: usize = 0;
    let mut num_err: usize = 0;
    let mut num_timeout: usize = 0;
    for entry in WalkDir::new(dir) {
        let entry = entry.expect("Should be able to walk");
        let path = entry.path();
        let extension = path.extension().and_then(OsStr::to_str);
        if let Some("yml") = extension {
            // get yaml file

            let docs = YamlLoader::load_from_str(&fs::read_to_string(path)?)?;
            let doc = &docs[0];
            //println!("{:?}", doc);

            let input_file = doc["input_files"].as_str().unwrap();
            //println!("Input file: {}", input_file);

            let mut safety_verdict = None;
            for property in doc["properties"].as_vec().unwrap() {
                let property_file = property["property_file"].as_str().unwrap();
                let expected_verdict = property["expected_verdict"].as_bool().unwrap();
                if property_file != "../../../properties/unreach-call.prp" {
                    return Err(anyhow!("Unexpected property file"));
                }
                safety_verdict = Some(expected_verdict);
            }
            let Some(safety_verdict) = safety_verdict else {
                return Err(anyhow!("Property file does not contain a safety verdict"));
            };

            let btor2_path_buf = path
                .parent()
                .expect("YAML file should have a parent")
                .join(Path::new(input_file));
            let btor2_path = btor2_path_buf.as_path();

            print!("\t{}: ", btor2_path.display());
            io::stdout().flush()?;
            match check(btor2_path) {
                Ok(result) => {
                    if let Some(result) = result {
                        if result {
                            if safety_verdict {
                                num_correct_true += 1;
                                println!("true")
                            } else {
                                println!("WRONG true");
                                num_wrong_true += 1;
                            }
                        } else if safety_verdict {
                            num_wrong_false += 1;
                            println!("WRONG false")
                        } else {
                            num_correct_false += 1;
                            println!("false")
                        }
                    } else {
                        num_timeout += 1;
                        println!("TIMEOUT");
                    }
                }
                Err(_) => {
                    num_err += 1;
                    println!("ERROR");
                }
            }
        }
    }
    println!(
        "Batch execution ended, {} correct true, {} correct false, {} wrong true, {} wrong false, {} errors, {} timeouts.",
        num_correct_true, num_correct_false, num_wrong_true, num_wrong_false, num_err, num_timeout
    );
    Ok(())
}

fn main() {
    // "btor2rs/examples/complex/goel-crafted"
    let mut args = env::args();
    // skip executable arg
    args.next();

    let Some(dir_name) = args.next() else {
        eprintln!("Directory not specified");
        return;
    };
    let dir_path = Path::new(&dir_name);

    let start = Instant::now();
    if let Err(err) = run(dir_path) {
        eprintln!("Batch fatal error: {:#}", err);
    }
    let elapsed = start.elapsed();
    println!("Batch execution took {:.3} s", elapsed.as_secs_f64());
}
