//! This is a testing harness for testing examples.

use std::process::{Output, Stdio};

#[derive(Debug)]
pub struct TestConfig {
    pub property: Option<String>,
    pub args: Vec<String>,
    pub input: String,
    pub release: bool,
}

impl TestConfig {
    pub fn new_property(property: &str) -> Self {
        Self {
            property: Some(String::from(property)),
            args: Vec::new(),
            input: String::new(),
            release: false,
        }
    }

    pub fn new_inherent() -> Self {
        Self {
            property: None,
            args: Vec::new(),
            input: String::new(),
            release: false,
        }
    }

    pub fn with_arg(mut self, arg: &str) -> Self {
        self.args.push(String::from(arg));
        self
    }

    pub fn with_input(mut self, input: &str) -> Self {
        self.input = String::from(input);
        self
    }

    pub fn with_release(mut self) -> Self {
        self.release = true;
        self
    }
}

fn print_example(example: &str, config: &TestConfig, stdout: &[u8], stderr: &[u8]) {
    println!(
        "Example '{}' with config {:?}\nStdout:\n{}\nStderr:\n{}",
        example,
        config,
        String::from_utf8_lossy(stdout),
        String::from_utf8_lossy(stderr)
    );
}

fn run_example(example: &str, config: &TestConfig, assume_inherent: bool) -> Output {
    let mut build = escargot::CargoBuild::new()
        .example(example)
        .manifest_path(env!("CARGO_MANIFEST_PATH"));

    if config.release {
        build = build.release();
    }

    let mut command = match build.run() {
        Ok(ok) => ok.command(),
        Err(err) => panic!(
            "Error preparing example '{}' with config {:?} as test: {}",
            example, config, err
        ),
    };

    command.arg("--batch");

    if let Some(property) = &config.property {
        command.arg("--property");
        command.arg(property);
    } else {
        command.arg("--inherent");
    }

    if assume_inherent {
        command.arg("--assume-inherent");
    }

    for arg in &config.args {
        command.arg(arg);
    }

    let mut spawned = match command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(ok) => ok,
        Err(err) => {
            panic!(
                "Error spawning example '{}' with config {:?} as test: {}",
                example, config, err
            )
        }
    };

    if !config.input.is_empty() {
        use std::io::Write;
        let mut stdin = spawned.stdin.take().expect("Should be able to take stdin");
        write!(stdin, "{}", config.input).unwrap();
    }

    match spawned.wait_with_output() {
        Ok(ok) => ok,
        Err(err) => panic!(
            "Error running example '{}' with config {:?} as test: {}",
            example, config, err
        ),
    }
}

pub fn test_example(example: &str, config: TestConfig, expected: &str) {
    let output = run_example(example, &config, false);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = &output.stderr;

    if !output.status.success() {
        print_example(example, &config, stdout.as_bytes(), stderr);
        panic!(
            "Running example did not terminate successfully, code: {:?}",
            output.status.code()
        );
    }

    let stdout = match stdout {
        std::borrow::Cow::Borrowed(stdout) => stdout,
        std::borrow::Cow::Owned(_) => {
            print_example(example, &config, stdout.as_bytes(), stderr);
            panic!("Running example returned non-UTF8 output",)
        }
    };

    if stdout != expected {
        print_example(example, &config, stdout.as_bytes(), stderr);
        panic!("Example stdout does not match expected");
    }

    // if there was a property specified, also test that it terminates successfully with assume inherent

    if config.property.is_some() {
        let output = run_example(example, &config, true);
        if !output.status.success() {
            print_example(example, &config, &output.stdout, &output.stderr);
            panic!(
                "When assuming inherent, example did not terminate successfully, code: {:?}",
                output.status.code()
            );
        }
    }
}

pub fn test_example_expect_panic(example: &str, config: TestConfig) {
    let output = run_example(example, &config, false);

    let stdout = &output.stdout;
    let stderr = &output.stderr;

    // rust sets exit code 101 on panic (this is only current behaviour,
    // but it seems unlikely to change and will at worst break the tests)
    if output.status.code() != Some(101) {
        print_example(example, &config, stdout, stderr);
        panic!(
            "Expected example to panic, but has status code: {:?}",
            output.status.code()
        );
    }
}
