use std::fs::{self, File};

use camino::Utf8PathBuf;
use clap::Parser;
use machine_check_hw::verify::VerifyResult;
use serde::Deserialize;

mod table;
mod xml;

#[derive(Parser, Clone, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(long)]
    pub xml: bool,

    #[arg(long)]
    pub table_name: Option<String>,

    pub dir: Utf8PathBuf,
}

struct TaskStats {
    expected_verdict: bool,
}

#[derive(Clone, Copy)]
enum RunResult {
    Correct(bool),
    Wrong(bool),
    Timeout,
    OutOfMemory,
    OtherError,
}

struct RunStats {
    name: String,
    result: Option<RunResult>,
    cpu_time: Option<f64>,
    wall_time: Option<f64>,
    memory: Option<u64>,
}

#[derive(Deserialize)]
struct Limits {
    #[serde(rename = "cpu-time")]
    cpu_time: Option<f64>,
    #[serde(rename = "cpu-cores")]
    cpu_cores: Option<u64>,
    #[serde(rename = "ram-size")]
    ram_size: Option<u64>,
}

#[derive(Deserialize)]
struct SystemInfo {
    #[serde(rename = "os-name")]
    os_name: Option<String>,
    #[serde(rename = "cpu-cores")]
    cpu_cores: Option<u64>,
    #[serde(rename = "cpu-frequency")]
    cpu_frequency: Option<u64>,
    #[serde(rename = "cpu-model")]
    cpu_model: Option<String>,
    #[serde(rename = "ram-size")]
    ram_size: Option<u64>,
}

#[derive(Clone, Copy, Deserialize)]
enum Tool {
    #[serde(rename = "machine-check")]
    MachineCheck = 0,
    #[serde(rename = "ABC")]
    Abc = 1,
    #[serde(rename = "AVR")]
    Avr = 2,
}

#[derive(Deserialize)]
struct TestConfig {
    name: Option<String>,

    tool: Tool,
    version: String,
    spec: String,

    options: Option<String>,

    #[serde(rename = "limits")]
    limits: Limits,

    #[serde(rename = "system-info")]
    system_info: SystemInfo,
}

fn main() {
    let args = Cli::parse();

    let test_config_path = args.dir.join("test-config.toml");

    let mut test_data_vec = Vec::new();

    if test_config_path.is_file() {
        test_data_vec.push(parse_test_data(test_config_path));
    } else {
        // look in the subdirectories
        for entry in fs::read_dir(&args.dir).expect("directory should be navigable") {
            let path = entry.expect("directory entry should be ok").path();
            if path.is_dir() {
                let test_config_path = Utf8PathBuf::try_from(path)
                    .expect("subdirectory path should be valid UTF-8")
                    .join("test-config.toml");
                if test_config_path.is_file() {
                    test_data_vec.push(parse_test_data(test_config_path));
                }
            }
        }
    }

    // sort like in the paper
    test_data_vec.sort_by(|a, b| {
        (a.0.tool as u32)
            .cmp(&(b.0.tool as u32))
            .then(a.0.options.cmp(&b.0.options))
    });

    if args.xml {
        for test_data in &test_data_vec {
            let test_config = &test_data.0;
            let tests = &test_data.1;
            let filename = test_config
                .name
                .clone()
                .unwrap_or_else(|| match test_config.tool {
                    Tool::MachineCheck => String::from("machine-check"),
                    Tool::Abc => String::from("abc"),
                    Tool::Avr => String::from("avr"),
                });

            let file = File::create(args.dir.join(format!("{}.xml", filename)))
                .expect("xml file should be creatable");
            xml::generate(test_config, tests).write(file).unwrap();
        }
    }

    table::print_table(args.table_name, &test_data_vec);
}

fn parse_test_data(test_config_path: Utf8PathBuf) -> (TestConfig, Vec<(TaskStats, RunStats)>) {
    let test_config =
        std::fs::read_to_string(test_config_path.clone()).expect("test config should be readable");
    let test_config: TestConfig =
        toml::from_str(&test_config).expect("test config should be valid TOML config");

    let test_config_dir = test_config_path.parent().unwrap();

    let test_spec_path = test_config_dir.join(Utf8PathBuf::from(&test_config.spec));

    let spec = std::fs::read_to_string(test_spec_path).expect("test spec should be readable");

    let exec_tests_dir = test_config_dir.join("tests");

    let mut tests = Vec::new();

    for line in spec.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // move into the appropriate folder
        let test_dir = exec_tests_dir.join(line);
        let expected_verdict_file = test_dir.join("expected-verdict");
        let expected_verdict = std::fs::read_to_string(expected_verdict_file)
            .expect("expected verdict should be readable");
        let expected_verdict = match expected_verdict.trim() {
            "false" => false,
            "true" => true,
            _ => panic!("Unknown expected verdict {:?}!", expected_verdict),
        };

        let task_stats = TaskStats { expected_verdict };

        let mut run_stats = RunStats {
            name: String::from(line),
            result: None,
            cpu_time: None,
            wall_time: None,
            memory: None,
        };

        let out_file = test_dir.join("out/out");
        let err_file = test_dir.join("out/err");
        let time_file = test_dir.join("out/time");
        if out_file.exists() && time_file.exists() {
            let out = match test_config.tool {
                Tool::MachineCheck | Tool::Abc => {
                    std::fs::read_to_string(out_file).expect("stdout file should be readable")
                }
                Tool::Avr => {
                    std::fs::read_to_string(err_file).expect("stderr file should be readable")
                }
            };
            let time = std::fs::read_to_string(time_file).expect("time file should be readable");
            if !out.is_empty() && !time.is_empty() {
                parse_run(&mut run_stats, &test_config, &task_stats, out, time);
            }
        }
        tests.push((task_stats, run_stats));
    }

    (test_config, tests)
}

fn parse_run(
    run_stats: &mut RunStats,
    test_config: &TestConfig,
    task_stats: &TaskStats,
    out: String,
    time: String,
) {
    parse_time(run_stats, time);

    match test_config.tool {
        Tool::MachineCheck => {
            let out: VerifyResult = serde_json::from_str(&out).unwrap();
            if let Some(exec) = out.exec {
                if let Ok(verdict) = exec.result {
                    run_stats.result = Some(create_run_result(task_stats, verdict));
                }
            }
        }
        Tool::Abc => {
            if out.contains("Property proved") {
                run_stats.result = Some(create_run_result(task_stats, true));
            } else if out.contains("was asserted") {
                run_stats.result = Some(create_run_result(task_stats, false));
            }
        }
        Tool::Avr => parse_avr_run(task_stats, run_stats, out),
    };

    if let (Some(cpu_time), Some(max_cpu_time)) = (run_stats.cpu_time, test_config.limits.cpu_time)
    {
        // make result timeout if it exceeded max cpu time
        if cpu_time > max_cpu_time {
            run_stats.result = Some(RunResult::Timeout);
        }
    }

    if run_stats.result.is_none() {
        if let (Some(memory), Some(max_memory)) = (run_stats.memory, test_config.limits.ram_size) {
            // determine yet undetermined errors to be OOM if memory was above 95% of permitted
            let conservative_memory = max_memory / 100 * 95;
            if memory > conservative_memory {
                run_stats.result = Some(RunResult::OutOfMemory);
            }
        }
    }
    // make result error if it was not determined
    if run_stats.result.is_none() {
        run_stats.result = Some(RunResult::OtherError);
    }
}

fn parse_avr_run(task_stats: &TaskStats, run_stats: &mut RunStats, out: String) {
    // find line that only has Result at first part
    let mut iter = out.lines();
    while let Some(line) = iter.next() {
        if let Some(first_part) = line.trim().split_ascii_whitespace().next() {
            if first_part == "Result" {
                // skip next line, line after that is important
                iter.next();
                let result_line = iter.next();
                if let Some(result_line) = result_line {
                    if let Some(first_part) = result_line.trim().split_ascii_whitespace().next() {
                        run_stats.result = Some(create_run_result(
                            task_stats,
                            match first_part {
                                "h" => true,
                                "v" => false,
                                _ => panic!("Unexpected result line {:?}", result_line),
                            },
                        ));
                    }
                }
            }
        }
    }
}

fn parse_time(run_stats: &mut RunStats, time: String) {
    // CPU time is user time + system time
    let mut user_time: Option<f64> = None;
    let mut system_time: Option<f64> = None;
    // we use /usr/bin/time for measurement
    for time_line in time.lines() {
        // split to key and value using colon followed by space,
        // as colons without space are also hour-minute-second delimiters
        let mut split = time_line.splitn(2, ": ");
        let Some(key) = split.next() else { continue };
        let Some(value) = split.next() else { continue };
        let key = key.trim();
        let value = value.trim();
        match key {
            "User time (seconds)" => user_time = Some(value.parse().unwrap()),
            "System time (seconds)" => system_time = Some(value.parse().unwrap()),
            "Elapsed (wall clock) time (h:mm:ss or m:ss)" => {
                // convert to seconds
                let mut total = 0.;
                for field in value.splitn(3, ':') {
                    total *= 60.;
                    total += field.parse::<f64>().unwrap();
                }
                run_stats.wall_time = Some(total);
            }
            "Maximum resident set size (kbytes)" => {
                // this is actually in kibibytes, convert to bytes
                run_stats.memory = Some(value.parse::<u64>().unwrap().checked_mul(1024).unwrap());
            }
            _ => (),
        }
    }
    // convert to CPU time if possible
    if let (Some(user_time), Some(system_time)) = (user_time, system_time) {
        run_stats.cpu_time = Some(user_time + system_time);
    }
}

fn create_run_result(task_stats: &TaskStats, conclusion: bool) -> RunResult {
    if conclusion == task_stats.expected_verdict {
        RunResult::Correct(conclusion)
    } else {
        RunResult::Wrong(conclusion)
    }
}
