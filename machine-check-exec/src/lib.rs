mod model_check;
mod precision;
mod proposition;
mod refinery;
mod space;

use log::{error, info, log_enabled, trace};
use machine_check_common::{ExecError, ExecResult};
use mck::mark::MarkMachine;

use clap::Parser;
use proposition::{Literal, PropTemp, PropU, PropUni, Proposition};
use refinery::Refinery;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    batch: bool,

    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[arg(long)]
    property: Option<String>,

    #[arg(long)]
    use_decay: bool,
}

pub fn run<M: MarkMachine>() {
    if let Err(err) = run_inner::<M>() {
        // log root error
        error!("{:#?}", err);
        // terminate with non-success code
        std::process::exit(-1);
    }
    // terminate successfully, the information is in stdout
}

fn run_inner<M: MarkMachine>() -> Result<ExecResult, anyhow::Error> {
    let args = Args::parse();
    // logging to stderr, stdout will contain the result in batch mode
    let filter_level = match args.verbose {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };
    env_logger::builder().filter_level(filter_level).init();

    info!("Starting verification.");

    let verification_result = verify::<M>(args.property.as_ref(), args.use_decay);

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

    // serialize the verification result to stdout
    serde_json::to_writer(std::io::stdout(), &verification_result)?;
    Ok(verification_result)
}

fn verify<M: MarkMachine>(property: Option<&String>, use_decay: bool) -> ExecResult {
    let mut refinery = Refinery::<M>::new(use_decay);
    let proposition = select_proposition(property);
    let result = match proposition {
        Ok(proposition) => refinery.verify(&proposition),
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
        // check AG[safe]
        Ok(Proposition::Negation(PropUni::new(Proposition::E(
            PropTemp::U(PropU {
                hold: Box::new(Proposition::Const(true)),
                until: Box::new(Proposition::Negation(PropUni::new(Proposition::Literal(
                    Literal::new(String::from("safe")),
                )))),
            }),
        ))))
    }
}
