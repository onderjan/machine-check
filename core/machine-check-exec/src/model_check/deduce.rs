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
        let culprit = deducer.deduce_end(0, &mut reasons)?;
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

impl<M: FullMachine> Deducer<'_, M> {
    /// Deduces the ending states of the culprit, after the ones already found.
    fn deduce_end(
        &mut self,
        subproperty_index: usize,
        reasons: &mut Vec<Reason>,
    ) -> Result<Culprit, ExecError> {
        trace!(
            "Deducing ending culprit states after {:?}, reasons {:?}",
            self.path,
            reasons
        );

        let subproperty_entry = self.property.subproperty_entry(subproperty_index);

        match &subproperty_entry.ty {
            PropertyType::Const(_) => panic!("Deduction should never reach const"),
            PropertyType::Atomic(atomic) => {
                // culprit ends here
                Ok(Culprit {
                    path: self.path.clone(),
                    atomic_property: atomic.clone(),
                })
            }
            PropertyType::Negation(inner) => self.deduce_end(*inner, reasons),
            PropertyType::BiLogic(op) => {
                let reason = reasons
                    .pop()
                    .expect("Deduction reasons should not be exhausted");
                let Reason::BiLogic(choice) = reason else {
                    panic!("Should deduce on binary logic operator");
                };

                let chosen_inner = match choice {
                    BiChoice::Left => op.a,
                    BiChoice::Right => op.b,
                };

                self.deduce_end(chosen_inner, reasons)
            }
            PropertyType::Next(op) => {
                let reason = reasons
                    .pop()
                    .expect("Deduction reasons should not be exhausted");
                let Reason::Next(next_state_id) = reason else {
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
            PropertyType::FixedPoint(op) => self.deduce_end(op.inner, reasons),
            PropertyType::FixedVariable(fixed_point_index) => {
                let reason = reasons
                    .pop()
                    .expect("Deduction reasons should not be exhausted");
                let Reason::FixedVariable(time) = reason else {
                    panic!("Should deduce on fixed variable");
                };

                // deduce on the variable from the given state
                // TODO: manage times correctly

                let current_state_id = *self.path.back().unwrap();

                let value = self
                    .getter
                    .property_checker()
                    .get_history(*fixed_point_index)
                    .before_time(time + 1, current_state_id)
                    .value;

                let CheckValue::Unknown(mut reasons) = value.clone() else {
                    panic!("Check value should be unknown when deducing from fixed point with state {}, time {}", current_state_id, time);
                };

                trace!(
                    "Deducing on new fixed point index {} with reasons {:?}",
                    fixed_point_index,
                    reasons
                );

                self.deduce_end(*fixed_point_index, &mut reasons)
            }
        }
    }
}
