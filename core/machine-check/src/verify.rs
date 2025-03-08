use crate::{ExecArgs, ExecError, ExecResult, FullMachine};
use log::{info, warn};
use machine_check_common::ExecStats;
use machine_check_exec::{Framework, PreparedProperty, Proposition, Strategy};

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
    if prop.is_none() != run_args.inherent {
        panic!("Expected exactly one of property or inherent");
    }
    verify_inner(system, &prop, run_args.assume_inherent, strategy)
}

fn verify_inner<M: FullMachine>(
    system: M,
    prop: &Option<Proposition>,
    assume_inherent: bool,
    strategy: Strategy,
) -> ExecResult {
    let abstract_system = <M::Abstr as mck::abstr::Abstr<M>>::from_concrete(system);

    /*let (property, assume_inherent) = match verification_type {
        VerificationType::Inherent => {
            // the inherent property is that there is no panic, i.e. AG[panic=0]
            let property = Proposition::inherent();
            (property, false)
        }
        VerificationType::Property(property) => (property, true),
    };*/

    if prop.is_none() && assume_inherent {
        return ExecResult {
            result: Err(ExecError::VerifiedInherentAssumed),
            stats: ExecStats::default(),
        };
    }
    // verify inherent property first
    let mut framework = Framework::<M>::new(abstract_system, &strategy);

    let inherent_property = PreparedProperty::new(&Proposition::inherent());

    match prop {
        Some(prop) => {
            if assume_inherent {
                warn!("Assuming that the inherent property holds. If it does not, the verification result will be unusable.");
            } else {
                info!("Verifying the inherent property first.");
                let inherent_result = framework.verify(&inherent_property, false);
                match inherent_result {
                    Ok(inherent_holds) => {
                        if inherent_holds {
                            // we are fine, ignore the inherent result stats
                            info!("The inherent property holds.");
                        } else {
                            // find the panic string
                            let Some(panic_str) = framework.find_panic_string() else {
                                panic!("Panic string should be found as inherent property does not hold");
                            };
                            return ExecResult {
                                result: Err(ExecError::InherentPanic(String::from(panic_str))),
                                stats: framework.info(),
                            };
                        }
                    }
                    Err(_) => {
                        // return the errors
                        return ExecResult {
                            result: inherent_result,
                            stats: framework.info(),
                        };
                    }
                }
            }

            info!("Verifying the given property.");
            let property = PreparedProperty::new(prop);

            // verify the property, assuming no panic can occur
            let result = framework.verify(&property, assume_inherent);

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
            info!("Verifying the inherent property.");
            let result = framework.verify(&inherent_property, false);
            match result {
                Ok(true) => info!("The inherent property holds."),
                Ok(false) => info!("The inherent property does not hold."),
                Err(_) => {}
            }
            ExecResult {
                result,
                stats: framework.info(),
            }
        }
    }
}
