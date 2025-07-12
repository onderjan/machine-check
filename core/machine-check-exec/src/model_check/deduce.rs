use core::panic;
use std::collections::VecDeque;

use machine_check_common::{
    check::{Culprit, Property},
    property::PropertyType,
    ExecError, StateId,
};
use mck::concr::FullMachine;

use crate::{model_check::PropertyChecker, space::StateSpace};

/// Deduces the culprit of unknown three-valued model-checking result.
pub(super) fn deduce_culprit<M: FullMachine>(
    checker: &PropertyChecker,
    space: &StateSpace<M>,
    property: &Property,
) -> Result<Culprit, ExecError> {
    //println!("Deducing culprit, checker: {:#?}", checker);

    // incomplete, compute culprit
    // it must start with one of the initial states
    for initial_index in space.initial_iter() {
        if checker
            .get_state_root_label(initial_index)
            .last_point()
            .value
            .is_unknown()
        {
            // unknown initial state, compute culprit from it
            let mut path = VecDeque::new();
            path.push_back(initial_index);
            let mut deducer = Deducer::<M> {
                checker,
                path,
                space,
                property,
            };
            let Deduction::Culprit(culprit) = deducer.deduce_end(0)? else {
                panic!("Deduction should give the culprit");
            };
            return Ok(culprit);
        }
    }

    unreachable!("no labelling culprit found");
}

struct Deducer<'a, M: FullMachine> {
    checker: &'a PropertyChecker,
    space: &'a StateSpace<M>,
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
    fn deduce_end(&mut self, subproperty_index: usize) -> Result<Deduction, ExecError> {
        //println!("Checker: {:#?}", self.checker);
        assert!(self
            .checker
            .get_state_label(subproperty_index, *self.path.back().unwrap())
            .last_point()
            .value
            .is_unknown());

        let subproperty_entry = self.property.subproperty_entry(subproperty_index);
        /*println!(
            "Deducing end for subproperty {:?} with path {:?}",
            subproperty_entry, self.path
        );*/

        match &subproperty_entry.ty {
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
                // the state should be unknown in p or q
                let state_index = *self.path.back().unwrap();
                let a_labelling = self.checker.get_state_label(op.a, state_index);
                let a_deduction = if a_labelling.last_point().value.is_unknown() {
                    let a_deduction = self.deduce_end(op.a)?;
                    if matches!(a_deduction, Deduction::Culprit(_)) {
                        return Ok(a_deduction);
                    }
                    Some(a_deduction)
                } else {
                    None
                };
                let b_labelling = self.checker.get_state_label(op.b, state_index);
                assert!(b_labelling.last_point().value.is_unknown());
                let b_deduction = self.deduce_end(op.b)?;
                if matches!(b_deduction, Deduction::Culprit(_)) {
                    return Ok(b_deduction);
                }
                // prefer the left deduction over the right one
                Ok(a_deduction.unwrap_or(b_deduction))
            }
            PropertyType::Next(op) => {
                let current_state = *self.path.back().unwrap();

                let labelling = self
                    .checker
                    .get_labelling(subproperty_index)
                    .get(&current_state)
                    .expect("Culprit state should have labelling");

                let next_state = *labelling
                    .last_point()
                    .next_states
                    .last()
                    .expect("Culprit state should have next state for next operator");

                assert_ne!(current_state, next_state);
                assert!(self.space.contains_edge(current_state.into(), next_state));

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
        }
    }

    /*fn deduce_end_next(&mut self, inner: usize, reason: StateId) -> Result<Deduction, ExecError> {
        //println!("Deducing end for path: {:?}", self.path);
        // lengthen by direct successor with unknown inner
        let path_back_index = *self.path.back().unwrap();

        for direct_successor_index in self.space.direct_successor_iter(path_back_index.into()) {
            /*println!(
                "Considering {} -> {}",
                path_back_index, direct_successor_index
            );*/

            if direct_successor_index != reason {
                //println!("Not the reason");
                continue;
            }

            let direct_successor_labelling =
                self.checker.get_state_label(inner, direct_successor_index);

            if direct_successor_labelling.is_unknown() {
                // add to path
                //println!("Unknown, adding to path");
                self.path.push_back(direct_successor_index);
                return self.deduce_end(inner);
            } else {
                //println!("Not unknown");
            }
        }
        panic!("no next state culprit found")
    }*/
}
