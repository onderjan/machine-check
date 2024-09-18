use crate::{ExecArgs, ExecError, ExecResult, FullMachine};
use log::{info, warn};
use machine_check_common::ExecStats;
use machine_check_exec::{Framework, Proposition, Strategy, VerificationType};

/// Verifies the given system with given arguments.
///
/// If verifying the inherent property, false is returned if it does not hold.
///
/// If verifying a standard property and the inherent property is not assumed,
/// it is verified first. If it does not hold, it is an execution error.
pub fn verify<M: FullMachine>(system: M, run_args: ExecArgs) -> ExecResult {
    let strategy = Strategy {
        naive_inputs: run_args.naive_inputs,
        use_decay: run_args.use_decay,
    };

    let prop = if let Some(property_str) = run_args.property {
        match Proposition::parse(&property_str) {
            Ok(prop) => Some(prop),
            Err(err) => {
                return ExecResult {
                    result: Err(err),
                    stats: ExecStats::default(),
                }
            }
        }
    } else {
        // check for inherent panics
        None
    };
    verify_inner(system, &prop, run_args.assume_inherent, strategy)
}

fn verify_inner<M: FullMachine>(
    system: M,
    prop: &Option<Proposition>,
    assume_inherent: bool,
    strategy: Strategy,
) -> ExecResult {
    let abstract_system = <M::Abstr as mck::abstr::Abstr<M>>::from_concrete(system);

    // verify inherent property first
    // if assumed, it would return `ExecError::VerifiedInherentAssumed`, short-circuit that
    // so that the framework is not even constructed, we will resolve it later
    let inherent_result = {
        if assume_inherent {
            ExecResult {
                result: Err(ExecError::VerifiedInherentAssumed),
                stats: ExecStats::default(),
            }
        } else {
            info!("Verifying the inherent property.");
            let mut framework =
                Framework::<M>::new(&abstract_system, VerificationType::Inherent, &strategy);
            let result = framework.verify();
            ExecResult {
                result,
                stats: framework.info(),
            }
        }
    };

    match prop {
        Some(prop) => {
            match inherent_result.result {
                Ok(_inherent_stats) => {
                    // we are fine, ignore the inherent result stats
                    info!("The inherent property holds.");
                }
                Err(ExecError::VerifiedInherentAssumed) => {
                    // assuming that inherent property holds is okay here
                    warn!("Assuming that the inherent property holds. If it does not, the verification result will be unusable.");
                }
                Err(_) => {
                    // return the other errors
                    return inherent_result;
                }
            }

            info!("Verifying the given property.");

            // create a new framework for the property checking so that running inherent verification and then assuming inherent
            // has the same logic as verifying a property without assuming inherent
            let mut framework = Framework::<M>::new(
                &abstract_system,
                VerificationType::Property(prop.clone()),
                &strategy,
            );
            // verify the property, assuming no panic can occur
            let result = framework.verify();

            // inform about the given property result if it was ok
            match result {
                Ok(true) => info!("The given property holds."),
                Ok(false) => info!("The given property does not hold."),
                Err(_) => {}
            }

            // also return framework stats
            ExecResult {
                result,
                stats: framework.info(),
            }
        }
        None => {
            match inherent_result.result {
                Ok(_) => {
                    // inform for parity with non-inherent property checking
                    info!("The inherent property holds.");
                    inherent_result
                }
                Err(ExecError::InherentPanic(panic_string)) => {
                    // inherent property does not hold
                    // log the panic string and return false instead
                    info!(
                        "The inherent property does not hold, panic string: '{}'",
                        panic_string
                    );
                    ExecResult {
                        result: Ok(false),
                        stats: inherent_result.stats,
                    }
                }
                Err(_) => inherent_result,
            }
        }
    }
}
