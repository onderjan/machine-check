/*use core::panic;
use std::{collections::VecDeque, ops::ControlFlow, u64};

use log::trace;
use machine_check_common::{
    check::{Culprit, Property},
    property::PropertyType,
    ExecError, StateId,
};
use mck::concr::FullMachine;

use crate::{
    model_check::{
        property_checker::{BiChoice, CheckChoice, CheckValue, LabellingCacher},
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

        let CheckValue::Unknown(choices) = timed.value else {
            continue;
        };
        // unknown initial state, compute culprit from it
        let mut path = VecDeque::new();
        path.push_back(initial_id);
        let deducer = Deducer::<M> {
            getter,
            path,
            property,
            subproperty_index: 0,
            choices,
            current_time: u64::MAX,
        };
        let culprit = deducer.deduce()?;
        trace!("Deduced culprit {:?}", culprit);
        return Ok(culprit);
    }

    unreachable!("Labelling culprit should start in initial states");
}

struct Deducer<'a, M: FullMachine> {
    getter: LabellingCacher<'a, M>,
    property: &'a Property,
    path: VecDeque<StateId>,
    subproperty_index: usize,
    choices: Vec<CheckChoice>,
    current_time: u64,
}

impl<M: FullMachine> Deducer<'_, M> {
    /// Deduces the culprit.
    fn deduce(mut self) -> Result<Culprit, ExecError> {
        loop {
            if let ControlFlow::Break(culprit) = self.deduce_iteration()? {
                return Ok(culprit);
            }
        }
    }

    /// Iterates on the deduction.
    fn deduce_iteration(&mut self) -> Result<ControlFlow<Culprit, ()>, ExecError> {
        trace!(
            "Deducing ending culprit states after {:?}, reasons {:?}",
            self.path,
            self.choices
        );

        let subproperty_entry = self.property.subproperty_entry(self.subproperty_index);

        self.subproperty_index = match &subproperty_entry.ty {
            PropertyType::Const(_) => panic!("Deduction should never reach const"),
            PropertyType::Atomic(atomic) => {
                // culprit ends here
                return Ok(ControlFlow::Break(Culprit {
                    path: self.path.clone(),
                    atomic_property: atomic.clone(),
                }));
            }
            PropertyType::Negation(inner) => {
                // just move to inner
                *inner
            }

            PropertyType::BiLogic(op) => {
                // find out the choice made
                let choice = self
                    .choices
                    .pop()
                    .expect("Deduction reasons should not be exhausted");
                let CheckChoice::BiLogic(choice) = choice else {
                    panic!("Should deduce on binary logic operator");
                };

                // move to the chosen
                match choice {
                    BiChoice::Left => op.a,
                    BiChoice::Right => op.b,
                }
            }
            PropertyType::Next(op) => {
                // find out the choice made
                let choice = self
                    .choices
                    .pop()
                    .expect("Deduction reasons should not be exhausted");
                let CheckChoice::Next(next_state_id) = choice else {
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

                // move to inner
                op.inner
            }
            PropertyType::FixedPoint(op) => {
                // just move to inner
                op.inner
            }
            PropertyType::FixedVariable(fixed_point_index) => {
                // find out the choice made
                let choice = self
                    .choices
                    .pop()
                    .expect("Deduction reasons should not be exhausted");
                let CheckChoice::FixedVariable(choice_time) = choice else {
                    panic!("Should deduce on fixed variable");
                };

                // ensure the choice has lesser time than current to ensure the deduction will finish
                assert!(choice_time < self.current_time);

                // replace the reasons with the reasons on the variable from the latest state and choice time
                let current_state_id = *self.path.back().unwrap();

                let value = self
                    .getter
                    .property_checker()
                    .get_history(*fixed_point_index)
                    .up_to_time(choice_time, current_state_id)
                    .value;

                let CheckValue::Unknown(choices) = value.clone() else {
                    panic!("Check value should be unknown when deducing from fixed point with state {}, time {}", current_state_id, choice_time);
                };

                self.choices = choices;
                self.current_time = choice_time;

                trace!(
                    "Deducing on new fixed point index {} with reasons {:?}",
                    fixed_point_index,
                    self.choices
                );

                // move to inner index
                *fixed_point_index
            }
        };
        Ok(ControlFlow::Continue(()))
    }
}
*/
