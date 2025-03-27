#![doc = include_str!("../README.md")]

mod args;
mod traits;
mod types;
mod verify;

use log::error;
use log::info;
use log::log_enabled;
use log::trace;
use machine_check_common::property::Property;
use machine_check_exec::Strategy;

use args::ProgramArgs;
pub use args::{ExecArgs, ExecStrategy};
pub use traits::Ext;
pub use types::{Bitvector, BitvectorArray, Signed, Unsigned};

/// Input to [`Machine`].
pub use ::mck::concr::Input;

/// State of [`Machine`].
pub use ::mck::concr::State;

/// Finite-state machine intended to be verifiable by **machine-check**.
///
/// To actually be verifiable by **machine-check**, further processing must be done by enclosing the structures
/// and [`Input`], [`State`], and [`Machine`] implementations within the [`machine_description`] macro.
///
pub use ::mck::concr::Machine;

use ::mck::concr::FullMachine;

/// Switch using a bitmask as scrutinee, useful for switching on processor instruction opcodes.
///
/// The switch is similar to a normal Rust match expression:
/// ```
/// use machine_check::{Bitvector, bitmask_switch};
/// let opcode = Bitvector::<6>::new(0b10_1101);
/// let mut foo = Bitvector::<2>::new(0);
/// let mut bar = Bitvector::<2>::new(0);
/// bitmask_switch!(opcode {
///    "00_----" => {}
///    "10_aabb" => {
///         foo = a;
///         bar = b;
///    }
///    "11_--cc" => {
///         foo = c;
///    }
///    _ => {}
/// });
/// assert_eq!(foo, Bitvector::new(3));
/// assert_eq!(bar, Bitvector::new(1));
/// ```
///
/// Unlike Rust match, the scrutinee must be [`Bitvector`], and the non-default choices are string literals
/// containing, for each bit of the bitvector, one of these:
/// - '0' or '1': the bit must match,
/// - dash ('-'): the bit can have any value (don't care),
/// - ASCII letter: same as don't care, but a new variable is created containing the bits with given letter.
/// - Underscore ('_') used to visually separate the bits.
///
/// Unlike Rust match, overlapping choices are not permitted, so that it is certain which arm is taken.
/// In case there is no default arm, there must be full coverage.
///
/// Currently, the macro cannot return values and there is no syntactic disjunction guarantee, i.e. that
/// exactly one of the arms is taken. This may change in the future.
///
///
pub use ::machine_check_macros::bitmask_switch;

/// Processes a module so that it can be used in **machine-check** verification.
///
/// To efficiently verify a system with **machine-check**, verification equivalents of the system that allow
/// more advanced reasoning (e.g. not caring about the value of some variable unless found to be necessary)
/// must be created, which is done by enclosing the system code in a module and applying this macro on it.
///
/// In practice, everything used in [`Machine`] must be processed by this macro. System construction,
/// however, can (and should) be done outside.
///
/// Note that, due to [a Rust issue](https://github.com/rust-lang/rust/issues/54726), procedural macros
/// currently cannot be used as inner attributes, so this is the currently recommended way of
/// using the macro:
/// ```
/// #[machine_check::machine_description]
/// mod machine_module {
///     // ... structs implementing Input, State, Machine ...
/// }
/// ```
///
/// The macro is currently rather limited in the subset of Rust code it can process, and errors may be cryptic.
/// Improvements are planned in the future. For now, the examples in the crate show code processable without errors.
///
pub use ::machine_check_macros::machine_description;

/// Executes machine-check with system environment arguments.
///
/// Is supposed to be used for simple systems that do not take arguments.
///
/// The system must implement [`Machine`]. The system structures and [`Input`], [`State`], and [`Machine`]
/// implementations must be enclosed within the [`machine_description`] macro, which processes them to enable
/// fast and efficient formal verification.
pub fn run<M: FullMachine>(system: M) -> ExecResult {
    let parsed_args = <ExecArgs as clap::Parser>::parse_from(std::env::args());
    execute(system, parsed_args)
}

/// Parses machine-check and user-defined arguments.
///
/// Returns arguments parsed to `machine-check` and system-specific argument definitions.
/// The arguments can be later used in [`execute`].
pub fn parse_args<A: clap::Args>(args: impl Iterator<Item = String>) -> (ExecArgs, A) {
    let parsed_args = <ProgramArgs<A> as clap::Parser>::parse_from(args);
    (parsed_args.run_args, parsed_args.system_args)
}

pub use ::machine_check_common::ExecError;
pub use ::machine_check_common::ExecResult;
pub use ::machine_check_common::ExecStats;

/// Runs **machine-check** with the given constructed system and parsed arguments.
///
/// Parsed arguments are used to run **machine-check**. Otherwise, this method behaves the same as [`run`].
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

    let strategy = Strategy {
        naive_inputs: matches!(exec_args.strategy, ExecStrategy::Naive),
        use_decay: matches!(exec_args.strategy, ExecStrategy::Decay),
    };

    // determine the property to verify
    let prop = if let Some(property_str) = exec_args.property {
        match Property::parse(&property_str) {
            Ok(prop) => Some(prop),
            Err(err) => {
                error!("Cannot construct the property: {}", err);
                return ExecResult {
                    result: Err(err),
                    stats: ExecStats::default(),
                };
            }
        }
    } else {
        // check for inherent panics
        None
    };
    if prop.is_none() && !exec_args.gui && !exec_args.inherent {
        panic!("Expected either a property or inherent verification");
    }

    let result = if exec_args.gui {
        // start the GUI instead of verifying
        ExecResult {
            result: Err(start_gui(system, prop, strategy)),
            stats: ExecStats::default(),
        }
    } else {
        info!("Starting verification.");

        let result = verify::verify(system, prop, exec_args.assume_inherent, strategy);

        if log_enabled!(log::Level::Trace) {
            trace!("Verification result: {:?}", result);
        }

        if log_enabled!(log::Level::Info) {
            // the result will be propagated, just inform that we ended somehow
            match result.result {
                Ok(_) => info!("Verification ended."),
                Err(_) => error!("Verification returned an error."),
            }
        }
        result
    };

    if !silent {
        if batch {
            // serialize the verification result to stdout
            if let Err(err) = serde_json::to_writer(std::io::stdout(), &result) {
                panic!("Could not serialize verification result: {:?}", err);
            }
        } else if !matches!(result.result, Err(ExecError::NoResult))
            && log_enabled!(log::Level::Info)
        {
            // print the verification result nicely
            let result_title = match &result.result {
                Ok(false) => "Result: DOES NOT HOLD",
                Ok(true) => "Result: HOLDS",
                Err(err) => &format!("Result: ERROR ({})", err),
            };

            let mut stats_cells: Vec<(&str, String)> = [
                ("Refinements", result.stats.num_refinements),
                ("Generated states", result.stats.num_generated_states),
                ("Final states", result.stats.num_final_states),
                (
                    "Generated transitions",
                    result.stats.num_generated_transitions,
                ),
                ("Final transitions", result.stats.num_final_transitions),
            ]
            .into_iter()
            .map(|(name, value)| (name, value.to_string()))
            .collect();

            if let Some(inherent_panic_message) = &result.stats.inherent_panic_message {
                stats_cells.push((
                    "Inherent panic message",
                    format!("{:?}", inherent_panic_message),
                ));
            }

            let inner_table_width = stats_cells
                .iter()
                .map(|(name, value)| format!("{}: {}", name, value).len())
                .max()
                .unwrap()
                .max(result_title.len());

            let result_title = format!(
                "|   {:^width$}   |",
                result_title,
                width = inner_table_width
            );
            let table_bar = format!("+{}+", "-".repeat(result_title.len().saturating_sub(2)));

            // the log is printed to stderr, follow it
            eprintln!("{}\n{}\n{}", table_bar, result_title, table_bar);
            for (name, value) in stats_cells {
                eprintln!(
                    "|  {}: {:>width$}  |",
                    name,
                    value,
                    width = inner_table_width - name.len()
                )
            }
            eprintln!("{}", table_bar);
        }
    }
    result
}

fn start_gui<M: FullMachine>(
    system: M,
    property: Option<Property>,
    strategy: Strategy,
) -> ExecError {
    // the GUI will, at best, return no result
    #[cfg(feature = "gui")]
    match machine_check_gui::run(system, property, strategy) {
        Ok(()) => ExecError::NoResult,
        Err(err) => err,
    }
    #[cfg(not(feature = "gui"))]
    {
        // make sure there is no warning about unused variables
        let _ = (system, property, strategy);
        ExecError::GuiError(String::from("The GUI feature was not enabled during build"))
    }
}

#[doc(hidden)]
pub mod mck {
    pub use mck::*;
}
