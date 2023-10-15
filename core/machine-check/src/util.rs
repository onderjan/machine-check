use std::process::Output;

use log::{info, log_enabled, trace};

pub fn log_process_output(process_type: &str, output: &Output) {
    if log_enabled!(log::Level::Trace) {
        let result = format!(
            "\n=== {} ===\nStatus: {:?}\nStdout:\n\"\"\"\n{}\n\"\"\"\nStderr:\n\"\"\"\n{}\n\"\"\"\n=== === ===",
            process_type,
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        trace!("{}", result);
    }
}

pub fn log_process_error_log(process_type: &str, error_log: &[u8]) {
    let result = format!(
        "\n=== {0} error log start ===\n{1}\n=== {0} error log end ===",
        process_type,
        String::from_utf8_lossy(error_log)
    );

    info!("{}", result);
}
