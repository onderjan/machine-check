use std::fs::File;

use camino::Utf8PathBuf;
use clap::Parser;
use machine_check::verify::VerifyResult;
use simple_xml_builder::XMLElement;

fn generate_xml_column(title: &str, value: Option<&str>) -> XMLElement {
    let mut xml_column = XMLElement::new("column");
    xml_column.add_attribute("title", title);
    if let Some(value) = value {
        xml_column.add_attribute("value", value);
    }
    xml_column
}

fn generate_xml_columns() -> XMLElement {
    let mut xml_columns = XMLElement::new("columns");
    xml_columns.add_child(generate_xml_column("status", None));
    xml_columns.add_child(generate_xml_column("cputime", None));
    xml_columns.add_child(generate_xml_column("walltime", None));
    xml_columns
}

struct TaskStats {
    expected_verdict: bool,
    max_cpu_time: f64,
}

struct RunStats {
    name: String,
    verdict: Option<bool>,
    cpu_time: Option<f64>,
    wall_time: Option<f64>,
    memory: Option<u64>,
}

fn generate_run(task_stats: TaskStats, run_stats: RunStats, max_memory: u64) -> XMLElement {
    let status = match run_stats.verdict {
        Some(verdict) => verdict.to_string(),
        None => String::from("ERROR"),
    };

    let category = match run_stats.verdict {
        Some(verdict) => {
            if verdict == task_stats.expected_verdict {
                String::from("correct")
            } else {
                String::from("wrong")
            }
        }
        None => String::from("error"),
    };

    let (mut status, category) = if let Some(cpu_time) = run_stats.cpu_time {
        // make status timeout if it exceeds max cpu time
        if cpu_time > task_stats.max_cpu_time {
            (String::from("TIMEOUT"), String::from("error"))
        } else {
            (status, category)
        }
    } else {
        // make status error if we do not have cpu time
        (String::from("ERROR"), String::from("error"))
    };

    if let Some(memory) = run_stats.memory {
        if status == "ERROR" {
            // determine yet undetermined errors to be OOM if memory was above 95% of permitted
            let conservative_memory = max_memory / 100 * 95;
            if memory > conservative_memory {
                status = String::from("OUT OF MEMORY");
            }
        }
    }

    let mut xml_run = XMLElement::new("run");
    xml_run.add_attribute("name", run_stats.name);
    xml_run.add_attribute("expectedVerdict", task_stats.expected_verdict);
    xml_run.add_child(generate_xml_column("status", Some(&status)));
    xml_run.add_child(generate_xml_column("category", Some(&category)));
    if let Some(cpu_time) = run_stats.cpu_time {
        xml_run.add_child(generate_xml_column(
            "cputime",
            Some(&format!("{:.6}s", cpu_time)),
        ));
    }
    if let Some(wall_time) = run_stats.wall_time {
        xml_run.add_child(generate_xml_column(
            "walltime",
            Some(&format!("{:.6}s", wall_time)),
        ));
    }
    if let Some(memory) = run_stats.memory {
        xml_run.add_child(generate_xml_column("memory", Some(&format!("{}B", memory))));
    }
    xml_run
}

fn generate_xml_systeminfo() -> XMLElement {
    let mut xml_systeminfo = XMLElement::new("systeminfo");

    let mut xml_os = XMLElement::new("os");
    xml_os.add_attribute("name", "(unknown)");
    xml_systeminfo.add_child(xml_os);

    let mut xml_cpu = XMLElement::new("cpu");
    xml_cpu.add_attribute("cores", "(unknown)");
    xml_cpu.add_attribute("frequency", "(unknown)");
    xml_cpu.add_attribute("model", "(unknown)");
    xml_systeminfo.add_child(xml_cpu);

    let mut xml_ram = XMLElement::new("ram");
    xml_ram.add_attribute("size", "(unknown)");
    xml_systeminfo.add_child(xml_ram);

    xml_systeminfo.add_child(XMLElement::new("environment"));

    xml_systeminfo
}

#[derive(Parser, Clone, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long)]
    pub spec: Utf8PathBuf,
    pub dir: Utf8PathBuf,

    #[arg(long)]
    pub max_cpu_time: Option<f64>,

    #[arg(long)]
    pub max_memory: Option<u64>,
}

fn main() {
    let args = Cli::parse();

    let max_cpu_time = args.max_cpu_time.unwrap_or(15. * 60.);
    let max_memory = args.max_memory.unwrap_or(16 * 1_000_000_000);

    let spec = std::fs::read_to_string(args.spec).unwrap();

    let exec_outputs_dir = args.dir;

    let exec_tests_dir = exec_outputs_dir.join("tests");
    let mut num_tests = 0;
    let mut num_outputs = 0;
    let mut num_errors = 0;
    let mut num_correct_false = 0;
    let mut num_correct_true = 0;
    let mut num_wrong_false = 0;
    let mut num_wrong_true = 0;
    let mut num_execution_failures = 0;
    let mut num_other_failures = 0;

    let mut avr = false;
    let mut abc = false;

    let mut xml_result = XMLElement::new("result");

    for line in spec.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        num_tests += 1;
        //println!("Line: {}", line);
        // move into the appropriate folder
        let test_dir = exec_tests_dir.join(line);
        if test_dir.is_dir() {
            num_outputs += 1;
        } else {
            continue;
        }
        let expected_verdict_file = test_dir.join("expected-verdict");
        let expected_verdict = std::fs::read_to_string(expected_verdict_file).unwrap();
        let expected_verdict = match expected_verdict.trim() {
            "false" => false,
            "true" => true,
            _ => panic!("Unknown expected verdict {:?}!", expected_verdict),
        };

        let task_stats = TaskStats {
            expected_verdict,
            max_cpu_time,
        };

        let mut run_stats = RunStats {
            name: String::from(line),
            verdict: None,
            cpu_time: None,
            wall_time: None,
            memory: None,
        };

        let out_file = test_dir.join("out/out");
        let time_file = test_dir.join("out/time");
        if out_file.exists() && time_file.exists() {
            let mut out = std::fs::read_to_string(out_file).unwrap();
            let time = std::fs::read_to_string(time_file).unwrap();

            if out.starts_with("AVR") {
                avr = true;
                let err_file = test_dir.join("out/err");
                out = std::fs::read_to_string(err_file).unwrap();
            } else if out.starts_with("ABC") {
                abc = true;
            }

            if !out.is_empty() && !time.is_empty() {
                // CPU time is user time + system time
                let mut user_time: Option<f64> = None;
                let mut system_time: Option<f64> = None;
                // we use /usr/bin/time for measurement
                for time_line in time.lines() {
                    // split to key and value using colon followed by space,
                    // as colons without space are also hour-minute-second delimiters
                    let mut split = time_line.splitn(2, ": ");
                    let Some(key) = split.next() else {continue};
                    let Some(value) = split.next() else {continue};
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
                            run_stats.memory =
                                Some(value.parse::<u64>().unwrap().checked_mul(1024).unwrap());
                        }
                        _ => (),
                    }
                }
                // convert to CPU time if possible
                if let (Some(user_time), Some(system_time)) = (user_time, system_time) {
                    run_stats.cpu_time = Some(user_time + system_time);
                }

                //println!("Result: {out}");
                if avr {
                    // find line that only has Result at first part
                    let mut iter = out.lines();
                    while let Some(line) = iter.next() {
                        if let Some(first_part) = line.trim().split_ascii_whitespace().next() {
                            if first_part == "Result" {
                                // skip next line, line after that is important
                                iter.next();
                                let result_line = iter.next();
                                if let Some(result_line) = result_line {
                                    if let Some(first_part) =
                                        result_line.trim().split_ascii_whitespace().next()
                                    {
                                        match first_part {
                                            "h" => run_stats.verdict = Some(true),
                                            "v" => run_stats.verdict = Some(false),
                                            _ => panic!("Unexpected result line {:?}", result_line),
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else if abc {
                    if out.contains("Property proved") {
                        run_stats.verdict = Some(true);
                    } else if out.contains("was asserted") {
                        run_stats.verdict = Some(false);
                    }
                } else {
                    let out: VerifyResult = serde_json::from_str(&out).unwrap();
                    if let Some(exec) = out.exec {
                        match exec.result {
                            Ok(exec_verdict) => {
                                run_stats.verdict = Some(exec_verdict);
                            }
                            Err(_) => {
                                num_errors += 1;
                            }
                        }
                    } else {
                        // execution failure
                        num_execution_failures += 1;
                    };
                }

                if let Some(verdict) = run_stats.verdict {
                    if verdict {
                        if expected_verdict {
                            num_correct_true += 1;
                        } else {
                            num_wrong_true += 1;
                        }
                    } else if expected_verdict {
                        num_wrong_false += 1;
                    } else {
                        num_correct_false += 1;
                    }
                }
            } else {
                num_other_failures += 1;
                // some problem was encountered
            }
        } else {
            num_other_failures += 1;
            // some problem was encountered
        }

        xml_result.add_child(generate_run(task_stats, run_stats, max_memory));
    }

    let tool_name = if abc {
        "ABC"
    } else if avr {
        "AVR"
    } else {
        "machine-check"
    };

    // add required attributes
    xml_result.add_attribute("tool", tool_name);
    xml_result.add_attribute("toolmodule", tool_name);
    xml_result.add_attribute("version", "(unknown)");
    xml_result.add_attribute("benchmarkname", "(unknown)");
    xml_result.add_attribute("starttime", "(unknown)");
    xml_result.add_attribute("date", "(unknown)");
    xml_result.add_attribute("generator", "machine-check-table");
    // not required by DTD, but table-generator fails without options
    xml_result.add_attribute("options", "");

    xml_result.add_child(generate_xml_columns());
    xml_result.add_child(generate_xml_systeminfo());

    println!(
        "Num tests: {}, num missing: {}",
        num_tests,
        num_tests - num_outputs
    );
    println!(
        "Num correct false: {}, true: {}, num wrong false: {}, true: {}, num errors: {}",
        num_correct_false, num_correct_true, num_wrong_false, num_wrong_true, num_errors
    );
    println!(
        "Num execution failures: {}, other failures: {}",
        num_execution_failures, num_other_failures
    );
    let total_correct = num_correct_false + num_correct_true;
    let total_wrong = num_wrong_false + num_wrong_true;
    let total_undetermined = num_tests - total_correct - total_wrong;
    println!(
        "Total correct: {}, wrong: {}, undetermined: {}",
        total_correct, total_wrong, total_undetermined
    );

    let file =
        File::create(exec_outputs_dir.join(format!("{}.xml", tool_name.to_lowercase()))).unwrap();
    xml_result.write(file).unwrap();
}
