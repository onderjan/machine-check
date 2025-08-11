use core::panic;
use std::collections::VecDeque;

use machine_check_common::{
    check::{Culprit, Property},
    property::PropertyType,
    ExecError, StateId,
};
use mck::concr::FullMachine;

use crate::{
    model_check::{
        property_checker::{BiChoice, LabellingCacher},
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
    //trace!("Deducing culprit, checker: {:#?}", checker);

    // incomplete, compute culprit
    // it must start with one of the initial states

    let getter = checker.last_getter(space);

    for initial_id in space.initial_iter() {
        let timed = getter.cache_and_get(0, initial_id)?;

        if timed.value.valuation.is_known() {
            continue;
        }
        // unknown initial state, compute culprit from it
        let mut path = VecDeque::new();
        path.push_back(initial_id);
        let mut deducer = Deducer::<M> {
            getter,
            path,
            property,
        };
        let Deduction::Culprit(culprit) = deducer.deduce_end(0)? else {
            panic!("Deduction should give the culprit");
        };
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
    fn deduce_end(&mut self, subproperty_index: usize) -> Result<Deduction, ExecError> {
        let last_state_id = *self.path.back().unwrap();
        assert!(self
            .getter
            .cache_and_get(subproperty_index, last_state_id)?
            .value
            .valuation
            .is_unknown());

        let subproperty_entry = self.property.subproperty_entry(subproperty_index);

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
                let a_timed = self.getter.cache_and_get(op.a, last_state_id)?;
                let b_timed = self.getter.cache_and_get(op.b, last_state_id)?;

                match LabellingCacher::<M>::choose_binary_op(op, &a_timed, &b_timed) {
                    BiChoice::Left => self.deduce_end(op.a),
                    BiChoice::Right => self.deduce_end(op.b),
                }
            }
            PropertyType::Next(op) => {
                let label = self
                    .getter
                    .cache_and_get(subproperty_index, last_state_id)?;

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
        }
    }
}
