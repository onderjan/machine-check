use std::path::Path;

use machine_check::VerifyResult;

fn main() {
    let spec = std::fs::read_to_string("examples/bv64-tasks.set").unwrap();
    let exec_outputs_dir = Path::new("exec/18173236/tests");
    let mut num_tests = 0;
    let mut num_outputs = 0;
    let mut num_errors = 0;
    let mut num_correct_false = 0;
    let mut num_correct_true = 0;
    let mut num_wrong_false = 0;
    let mut num_wrong_true = 0;
    let mut num_execution_failures = 0;
    let mut num_other_failures = 0;

    for line in spec.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        num_tests += 1;
        //println!("Line: {}", line);
        // move into the appropriate folder
        let output_dir = exec_outputs_dir.join(line);
        if output_dir.is_dir() {
            num_outputs += 1;
        } else {
            continue;
        }
        let expected_verdict_file = output_dir.join("expected-verdict");
        let expected_verdict = std::fs::read_to_string(expected_verdict_file).unwrap();
        let expected_verdict = match expected_verdict.trim() {
            "false" => false,
            "true" => true,
            _ => panic!("Unknown expected verdict {:?}!", expected_verdict),
        };
        let out_file = output_dir.join("out/out");
        if !out_file.exists() {
            num_other_failures += 1;
            // some problem was encountered
            continue;
        }
        let out = std::fs::read_to_string(out_file).unwrap();
        if out.is_empty() {
            num_other_failures += 1;
            // some problem was encountered
            continue;
        }
        //println!("Result: {out}");
        let out: VerifyResult = serde_json::from_str(&out).unwrap();
        let Some(exec) = out.exec else {
            // execution failure
            num_execution_failures += 1;
            continue;
        };
        match exec.result {
            Ok(verdict) => {
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
            Err(err) => {
                num_errors += 1;
                println!("Execution error: {}", err);
            }
        }
    }
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
}
