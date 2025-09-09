use core::panic;
use std::collections::VecDeque;

use log::trace;
use machine_check_common::{
    check::{Culprit, Property},
    property::PropertyType,
    ExecError, StateId,
};
use mck::concr::FullMachine;

use crate::{
    model_check::{
        property_checker::{BiChoice, CheckValue, LabellingCacher, Reason},
        PropertyChecker,
    },
    space::StateSpace,
};

/// Deduces the culprit of unknown three-valued model-checking result.
pub(super) fn deduce_culprit<M: FullMachine>(
    checker: &PropertyChecker,
    space: &StateSpace<M>,
    property: &Property,
) -> Result<Culprit, ExecError> {
    trace!("Deducing culprit");

    // incomplete, compute culprit
    // it must start with one of the initial states

    let getter = checker.last_getter(space);

    for initial_id in space.initial_iter() {
        let timed = getter.compute_latest_timed(0, initial_id)?;

        let CheckValue::Unknown(mut reasons) = timed.value else {
            continue;
        };
        // unknown initial state, compute culprit from it
        let mut path = VecDeque::new();
        path.push_back(initial_id);
        let mut deducer = Deducer::<M> {
            getter,
            path,
            property,
        };
        let Deduction::Culprit(culprit) = deducer.deduce_end(0, &mut reasons)? else {
            panic!("Deduction should give the culprit");
        };
        trace!("Deduced culprit {:?}", culprit);
        return Ok(culprit);
    }

    unreachable!("Labelling culprit should start in initial states");
}

struct Deducer<'a, M: FullMachine> {
    getter: LabellingCacher<'a, M>,
    property: &'a Property,
    path: VecDeque<StateId>,
}

#[derive(Debug)]
struct FixedPointDeduction {
    path: VecDeque<StateId>,
    variable: usize,
}

#[derive(Debug)]
enum Deduction {
    Culprit(Culprit),
    FixedPoint(FixedPointDeduction),
}

impl<M: FullMachine> Deducer<'_, M> {
    /// Deduces the ending states of the culprit, after the ones already found.
    fn deduce_end(
        &mut self,
        subproperty_index: usize,
        reasons: &mut Vec<Reason>,
    ) -> Result<Deduction, ExecError> {
        trace!("Deducing ending culprit states after {:?}", self.path);
        /*let current_state_id = *self.path.back().unwrap();
        assert!(self
            .getter
            .compute_latest_timed(subproperty_index, current_state_id)?
            .value
            .is_unknown());*/

        let subproperty_entry = self.property.subproperty_entry(subproperty_index);

        let reason = reasons
            .pop()
            .expect("Deduction reasons should not be exhausted");

        trace!(
            "Reason {:?}, Subproperty entry {:?}",
            reason,
            subproperty_entry,
        );

        let ty = &subproperty_entry.ty;

        match reason {
            Reason::Atomic => {
                let PropertyType::Atomic(atomic) = ty else {
                    panic!("Should deduce on atomic property");
                };

                // culprit ends here
                Ok(Deduction::Culprit(Culprit {
                    path: self.path.clone(),
                    atomic_property: atomic.clone(),
                }))
            }
            Reason::Negation => {
                let PropertyType::Negation(inner) = ty else {
                    panic!("Should deduce on negation operator");
                };

                self.deduce_end(*inner, reasons)
            }
            Reason::BiLogic(bi_choice) => {
                let PropertyType::BiLogic(op) = ty else {
                    panic!("Should deduce on binary logic operator");
                };

                let chosen_inner = match bi_choice {
                    BiChoice::Left => op.a,
                    BiChoice::Right => op.b,
                };

                self.deduce_end(chosen_inner, reasons)
            }
            Reason::Next(next_state_id) => {
                let PropertyType::Next(op) = ty else {
                    panic!("Should deduce on next operator");
                };

                // sanity assertion
                let current_state_id = *self.path.back().unwrap();
                assert!(self
                    .getter
                    .space()
                    .contains_edge(current_state_id.into(), next_state_id));

                // add state to path
                self.path.push_back(next_state_id);

                self.deduce_end(op.inner, reasons)
            }
            Reason::FixedPoint => {
                let PropertyType::FixedPoint(op) = ty else {
                    panic!("Should deduce on fixed point");
                };

                self.deduce_end(op.inner, reasons)
            }
            Reason::FixedVariable => {
                let PropertyType::FixedVariable(fixed_point_index) = ty else {
                    panic!("Should deduce on fixed variable");
                };

                // deduce on the variable from the given state
                // TODO: manage times correctly

                let current_state_id = *self.path.back().unwrap();
                let timed = self
                    .getter
                    .compute_latest_timed(*fixed_point_index, current_state_id)?;

                let CheckValue::Unknown(mut reasons) = timed.value else {
                    panic!("Check value should be unknown when deducing from fixed point");
                };

                self.deduce_end(*fixed_point_index, &mut reasons)

                //self.deduce_end(*fixed_point_index, reasons)
            }
        }

        /*match &subproperty_entry.ty {
            PropertyType::Const(_) => {
                // never ends in const
                panic!("const should never be the labelling culprit")
            }
            PropertyType::Atomic(literal) => {
                // culprit ends here
                Ok(Deduction::Culprit(Culprit {
                    path: self.path.clone(),
                    atomic_property: literal.clone(),
                }))
            }
            PropertyType::Negation(inner) => {
                // propagate to inner
                self.deduce_end(*inner)
            }
            PropertyType::BiLogic(op) => {
                let a_timed = self.getter.compute_latest_timed(op.a, last_state_id)?;
                let b_timed = self.getter.compute_latest_timed(op.b, last_state_id)?;

                match LabellingCacher::<M>::choose_binary_op(op, &a_timed, &b_timed) {
                    BiChoice::Left => self.deduce_end(op.a),
                    BiChoice::Right => self.deduce_end(op.b),
                }
            }
            PropertyType::Next(op) => {
                let label = self
                    .getter
                    .compute_latest_timed(subproperty_index, last_state_id)?;

                let next_state = *label
                    .value
                    .next_states
                    .last()
                    .expect("Culprit state should have next state for next operator");

                assert_ne!(last_state_id, next_state);
                assert!(self
                    .getter
                    .space()
                    .contains_edge(last_state_id.into(), next_state));

                self.path.push_back(next_state);

                self.deduce_end(op.inner)
            }
            PropertyType::FixedPoint(op) => {
                loop {
                    let deduction = self.deduce_end(op.inner)?;
                    match deduction {
                        Deduction::Culprit(_) => break Ok(deduction),
                        Deduction::FixedPoint(deduction) => {
                            if deduction.variable != subproperty_index {
                                // not our variable, break
                                break Ok(Deduction::FixedPoint(deduction));
                            }
                            // our variable, update path and loop
                            self.path = deduction.path;
                        }
                    }
                }
            }
            PropertyType::FixedVariable(variable) => {
                // return fixed-point deduction
                Ok(Deduction::FixedPoint(FixedPointDeduction {
                    path: self.path.clone(),
                    variable: *variable,
                }))
            }
        }*/
    }
}
