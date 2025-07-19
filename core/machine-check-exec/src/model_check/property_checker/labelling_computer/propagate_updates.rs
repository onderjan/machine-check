use std::collections::BTreeSet;

use log::trace;
use machine_check_common::{property::PropertyType, StateId};

use crate::{model_check::property_checker::labelling_computer::LabellingComputer, FullMachine};

impl<M: FullMachine> LabellingComputer<'_, M> {
    pub(super) fn propagate_updates(
        &mut self,
        subproperty_index: usize,
        partial_updates: &BTreeSet<StateId>,
    ) {
        let subproperty_entry = self.property.subproperty_entry(subproperty_index);

        trace!(
            "Propagating down to subproperty {} with partial updates {:?}",
            subproperty_index,
            partial_updates
        );
        let computation = Self::computation_mut(&mut self.computations, subproperty_index);
        computation.updated.extend(partial_updates);

        let mut add_updates = BTreeSet::new();

        match &subproperty_entry.ty {
            PropertyType::Const(_) | PropertyType::Atomic(_) => {
                // nothing to propagate
            }
            PropertyType::Negation(inner) => {
                self.propagate_updates(*inner, partial_updates);
                let inner_computation = Self::computation(&self.computations, *inner);
                add_updates.extend(inner_computation.updated.iter().copied());
            }
            PropertyType::BiLogic(op) => {
                self.propagate_updates(op.a, partial_updates);
                self.propagate_updates(op.b, partial_updates);

                let a_computation = Self::computation(&self.computations, op.a);
                add_updates.extend(a_computation.updated.iter().copied());
                let b_computation = Self::computation(&self.computations, op.b);
                add_updates.extend(b_computation.updated.iter().copied());
            }
            PropertyType::Next(op) => {
                let mut next_partial_updates = BTreeSet::new();
                for state_id in partial_updates.iter().copied() {
                    for successor_id in self.space.direct_successor_iter(state_id.into()) {
                        next_partial_updates.insert(successor_id);
                    }
                }
                self.propagate_updates(op.inner, &next_partial_updates);

                let inner_computation = Self::computation(&self.computations, op.inner);
                for state_id in inner_computation.updated.iter().copied() {
                    for predecessor_id in self.space.direct_predecessor_iter(state_id.into()) {
                        if let Ok(predecessor_id) = StateId::try_from(predecessor_id) {
                            add_updates.insert(predecessor_id);
                        }
                    }
                }
            }
            PropertyType::FixedPoint(op) => {
                self.propagate_updates(op.inner, partial_updates);

                let inner_computation = Self::computation(&self.computations, op.inner);
                add_updates.extend(inner_computation.updated.iter().copied());
            }
            PropertyType::FixedVariable(fixed_point_index) => {
                // nothing to propagate
                let fixed_variable_computation =
                    Self::computation(&self.computations, *fixed_point_index);
                add_updates.extend(fixed_variable_computation.updated.iter().copied());
            }
        };

        let computation = Self::computation_mut(&mut self.computations, subproperty_index);
        computation.updated.extend(add_updates);
    }
}
