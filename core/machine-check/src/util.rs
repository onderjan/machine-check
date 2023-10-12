use std::process::Output;

use log::{log_enabled, trace};

pub fn log_process_output(output: &Output) {
    if log_enabled!(log::Level::Trace) {
        trace!("Status: {:?}", output.status);
        trace!(
            "Stdout:\n\"\"\"\n{}\n\"\"\"",
            String::from_utf8_lossy(&output.stdout)
        );
        trace!(
            "Stderr:\n\"\"\"\n{}\n\"\"\"",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
