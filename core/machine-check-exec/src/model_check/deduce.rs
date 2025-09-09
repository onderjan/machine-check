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
        trace!("Deducing ending culprit states after {:?}", self.path);

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
                Ok(Culprit {
                    path: self.path.clone(),
                    atomic_property: atomic.clone(),
                })
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
            }
        }
    }
}
