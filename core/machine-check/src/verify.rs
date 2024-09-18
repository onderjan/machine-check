use crate::{ExecArgs, ExecResult, FullMachine};
use machine_check_exec::{Framework, Proposition, Strategy};

/// Executes machine-check with parsed arguments.
///
/// The arguments can be parsed using [`parse_args`].

/// Verifies the given system with given arguments.
pub fn verify<M: FullMachine>(system: M, run_args: ExecArgs) -> ExecResult {
    let strategy = Strategy {
        naive_inputs: run_args.naive_inputs,
        use_decay: run_args.use_decay,
    };
    let mut framework = Framework::<M>::new(system, strategy);

    let result = if let Some(property_str) = run_args.property {
        Proposition::parse(&property_str).map(Some)
    } else {
        // check for inherent panics without really checking a property, use constant true
        Ok(None)
    };
    let result = result
        .and_then(|proposition| framework.verify_property(&proposition, run_args.assume_inherent));

    ExecResult {
        result,
        stats: framework.info(),
    }
}
