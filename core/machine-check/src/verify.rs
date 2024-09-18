use crate::{ExecArgs, ExecError, ExecResult, FullMachine};
use log::{info, warn};
use machine_check_exec::{Framework, Proposition, Strategy};

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
    let mut framework = Framework::<M>::new(system, strategy);

    let result = if let Some(property_str) = run_args.property {
        Proposition::parse(&property_str).map(Some)
    } else {
        // check for inherent panics without really checking a property, use constant true
        Ok(None)
    };
    let result = result.and_then(|proposition| {
        verify_inner(&mut framework, &proposition, run_args.assume_inherent)
    });

    ExecResult {
        result,
        stats: framework.info(),
    }
}

fn verify_inner<M: FullMachine>(
    framework: &mut Framework<M>,
    prop: &Option<Proposition>,
    assume_inherent: bool,
) -> Result<bool, ExecError> {
    // verify inherent property first if not assumed
    let inherent_result = if assume_inherent {
        None
    } else {
        info!("Verifying the inherent property.");
        Some(framework.verify_inherent())
    };

    match prop {
        Some(prop) => {
            if let Some(inherent_result) = inherent_result {
                // return the inherent error if necessary
                inherent_result?;
            } else {
                warn!("Assuming that the inherent property holds. If it does not, the verification result will be unusable.");
            }

            // inherent property holds
            // verify the property, assuming no panic can occur
            framework.verify_property(prop)
        }
        None => {
            // ensure that we have verified the inherent property
            let Some(inherent_result) = inherent_result else {
                return Err(ExecError::VerifiedInherentAssumed);
            };

            if let Err(ExecError::InherentPanic(panic_string)) = inherent_result {
                // inherent property does not hold
                // log the panic string and return false
                info!(
                    "Inherent property does not hold, panic string: '{}'",
                    panic_string
                );
                return Ok(false);
            }
            // inherent property holds
            // return true
            inherent_result.map(|_| true)
        }
    }
}
