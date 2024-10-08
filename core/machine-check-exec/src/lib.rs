#![doc = include_str!("../README.md")]

mod framework;
mod model_check;
mod precision;
mod proposition;
mod space;

use log::{error, info, log_enabled, trace};
use machine_check_common::ExecResult;
use mck::concr::FullMachine;

use clap::{ArgGroup, Args, Parser};
use framework::{Framework, Strategy};
use proposition::Proposition;

/// Arguments for executing machine-check.
#[derive(Parser, Debug)]
#[clap(group(ArgGroup::new("property-group")
.required(true)
.multiple(true)
.args(&["property", "inherent"]),
))]
#[clap(group(ArgGroup::new("verbosity-group")
.required(false)
.multiple(false)
.args(&["silent", "verbose"]),
))]
#[clap(group(ArgGroup::new("assume-inherent-group")
.required(false)
.multiple(false)
.conflicts_with("inherent")
.args(&["assume_inherent"]),
))]
pub struct ExecArgs {
    #[arg(long)]
    pub silent: bool,

    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    #[arg(long)]
    pub batch: bool,

    #[arg(long)]
    pub property: Option<String>,

    #[arg(long)]
    pub inherent: bool,

    // experimental flags
    #[arg(long)]
    pub naive_inputs: bool,
    #[arg(long)]
    pub use_decay: bool,
    #[arg(long)]
    pub assume_inherent: bool,
}

#[derive(Parser, Debug)]
struct ProgramArgs<A: Args> {
    #[clap(flatten)]
    run_args: ExecArgs,
    #[clap(flatten)]
    system_args: A,
}

/// Parses machine-check and user-defined arguments.
///
/// Returns arguments parsed to `machine-check` and system-specific argument definitions.
/// The arguments can be later used in [`execute`].
pub fn parse_args<A: Args>(args: impl Iterator<Item = String>) -> (ExecArgs, A) {
    let parsed_args = ProgramArgs::<A>::parse_from(args);
    (parsed_args.run_args, parsed_args.system_args)
}

/// Executes machine-check with system environment arguments.
///
/// Is supposed to be used for simple systems that do not take arguments.
pub fn run<M: FullMachine>(system: M) -> ExecResult {
    let parsed_args = ExecArgs::parse_from(std::env::args());
    execute(system, parsed_args)
}

/// Executes machine-check with parsed arguments.
///
/// The arguments can be parsed using [`parse_args`].
pub fn execute<M: FullMachine>(system: M, exec_args: ExecArgs) -> ExecResult {
    // logging to stderr, stdout will contain the result in batch mode
    let silent = exec_args.silent;
    let batch = exec_args.batch;
    let mut filter_level = match exec_args.verbose {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };
    if silent {
        filter_level = log::LevelFilter::Off;
    }

    // initialize logger, but do not panic if it was already initialized
    let _ = env_logger::builder().filter_level(filter_level).try_init();

    info!("Starting verification.");

    let verification_result = verify(system, exec_args);

    if log_enabled!(log::Level::Trace) {
        trace!("Verification result: {:?}", verification_result);
    }

    if log_enabled!(log::Level::Info) {
        // the result will be propagated, just inform that we ended somehow
        match verification_result.result {
            Ok(_) => info!("Verification ended."),
            Err(_) => error!("Verification failed."),
        }
    }

    if !silent {
        if batch {
            // serialize the verification result to stdout
            if let Err(err) = serde_json::to_writer(std::io::stdout(), &verification_result) {
                panic!("Could not serialize verification result: {:?}", err);
            }
        } else {
            // TODO: nicer result printing
            info!("Verification result: {:?}", verification_result);
        }
    }
    verification_result
}

/// Verifies the given system with given arguments.
fn verify<M: FullMachine>(system: M, run_args: ExecArgs) -> ExecResult {
    let strategy = Strategy {
        naive_inputs: run_args.naive_inputs,
        use_decay: run_args.use_decay,
    };
    let mut refinery = Framework::<M>::new(system, strategy);

    let result = if let Some(property_str) = run_args.property {
        Proposition::parse(&property_str).map(Some)
    } else {
        // check for inherent panics without really checking a property, use constant true
        Ok(None)
    };
    let result = result
        .and_then(|proposition| refinery.verify_property(&proposition, run_args.assume_inherent));

    ExecResult {
        result,
        stats: refinery.info(),
    }
}
