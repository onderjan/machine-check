use crate::{ExecError, ExecResult, FullMachine};
use log::{info, warn};
use machine_check_common::{check::PreparedProperty, property::Property, ExecStats};
use machine_check_exec::{Framework, Strategy};

/// Verifies the given system with given arguments.
///
/// If verifying the inherent property, false is returned if it does not hold.
///
/// If verifying a standard property and the inherent property is not assumed,
/// it is verified first. If it does not hold, it is an execution error.
pub fn verify<M: FullMachine>(
    system: M,
    prop: Option<Property>,
    assume_inherent: bool,
    strategy: Strategy,
) -> ExecResult {
    let abstract_system = <M::Abstr as mck::abstr::Abstr<M>>::from_concrete(system);

    // Short-circuit error on assumption of the inherent property that we are trying to verify.
    if prop.is_none() && assume_inherent {
        return ExecResult {
            result: Err(ExecError::VerifiedInherentAssumed),
            stats: ExecStats::default(),
        };
    }

    // Construct the framework.
    let mut framework = Framework::<M>::new(abstract_system, strategy);

    // Verify the inherent property first if not assumed.
    let inherent_result = if assume_inherent {
        None
    } else {
        if prop.is_some() {
            info!("Verifying the inherent property first.");
        } else {
            info!("Verifying the inherent property.");
        }
        let inherent_property = PreparedProperty::new(Property::inherent());
        Some(framework.verify(&inherent_property))
    };

    let Some(property) = prop else {
        // Inherent property verification only.
        // Print info and return the inherent property verification result.
        // The property should be verified as the short-circuit was done previously.
        return ExecResult {
            result: inherent_result.expect("Inherent property should not be assumed"),
            stats: framework.info(),
        };
    };

    // Standard property verification.

    if let Some(inherent_result) = inherent_result {
        match inherent_result {
            Ok(true) => {
                // Inherent holds, we can continue.
                info!("The inherent property holds, proceeding to the given property.");
            }
            Ok(false) => {
                // inherent
                return ExecResult {
                    result: Err(ExecError::InherentPanic),
                    stats: framework.info(),
                };
            }
            Err(_) => {
                // return the errors
                return ExecResult {
                    result: inherent_result,
                    stats: framework.info(),
                };
            }
        }
    } else {
        warn!("Assuming that the inherent property holds. If it does not, the verification result will be unusable.");
    }

    info!("Verifying the given property.");
    let property = PreparedProperty::new(property);

    // verify the property, assuming no panic can occur
    let result = framework.verify(&property);

    // also return framework stats
    ExecResult {
        result,
        stats: framework.info(),
    }
}
