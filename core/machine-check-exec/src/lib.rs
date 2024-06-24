#![doc = include_str!("../README.md")]

mod model_check;
mod precision;
mod proposition;
mod refinery;
mod space;

use log::{error, info, log_enabled, trace};
use machine_check_common::{ExecError, ExecResult};
use mck::concr::FullMachine;

use clap::{ArgGroup, Args, Parser};
use proposition::Proposition;
use refinery::{Refinery, Settings};

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
pub struct RunArgs {
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
}

#[derive(Parser, Debug)]
struct ProgramArgs<A: Args> {
    #[clap(flatten)]
    run_args: RunArgs,
    #[clap(flatten)]
    system_args: A,
}

pub fn parse_args<A: Args>(args: impl Iterator<Item = String>) -> (RunArgs, A) {
    let parsed_args = ProgramArgs::<A>::parse_from(args);
    (parsed_args.run_args, parsed_args.system_args)
}

pub fn run<M: FullMachine>(system: M) -> ExecResult {
    let parsed_args = RunArgs::parse_from(std::env::args());
    run_with_parsed_args(system, parsed_args)
}

pub fn run_with_parsed_args<M: FullMachine>(system: M, run_args: RunArgs) -> ExecResult {
    match run_inner(system, run_args) {
        Ok(ok) => ok,
        Err(err) => {
            // log root error
            error!("{:#?}", err);
            // terminate with non-success code
            std::process::exit(-1);
        }
    }
}

fn run_inner<M: FullMachine>(system: M, run_args: RunArgs) -> Result<ExecResult, anyhow::Error> {
    // logging to stderr, stdout will contain the result in batch mode
    let silent = run_args.silent;
    let batch = run_args.batch;
    let mut filter_level = match run_args.verbose {
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

    let verification_result = verify(system, run_args);

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
            serde_json::to_writer(std::io::stdout(), &verification_result)?;
        } else {
            // TODO: nicer result printing
            info!("Verification result: {:?}", verification_result);
        }
    }
    Ok(verification_result)
}

fn verify<M: FullMachine>(system: M, run_args: RunArgs) -> ExecResult {
    let settings = Settings {
        naive_inputs: run_args.naive_inputs,
        use_decay: run_args.use_decay,
    };
    let mut refinery = Refinery::<M>::new(system, settings);
    let proposition = select_proposition(run_args.property.as_ref());
    let result = match proposition {
        Ok(proposition) => refinery.verify_property(&proposition),
        Err(err) => Err(err),
    };

    ExecResult {
        result,
        stats: refinery.info(),
    }
}

fn select_proposition(property: Option<&String>) -> Result<Proposition, ExecError> {
    if let Some(property_str) = property {
        Proposition::parse(property_str)
    } else {
        // check for inherent panics without really checking a property, use constant true
        Ok(Proposition::Const(true))
    }
}
