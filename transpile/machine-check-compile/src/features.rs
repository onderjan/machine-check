use std::process::Command;

fn features() -> Vec<&'static str> {
    // add features
    let mut features = Vec::new();
    if cfg!(feature = "gui") {
        features.push("gui");
    }
    if cfg!(feature = "Zdual_interval") {
        features.push("Zdual_interval");
    }
    features
}

pub(super) fn add_rustc_features(build_command: &mut Command) {
    for feature in features() {
        build_command
            .arg("--cfg")
            .arg(format!("feature=\"{}\"", feature));
    }
}

pub(super) fn add_cargo_features(build_command: &mut Command) {
    let features = features();
    if !features.is_empty() {
        let features = features.join(",");
        build_command.arg("--features").arg(features);
    }
}
