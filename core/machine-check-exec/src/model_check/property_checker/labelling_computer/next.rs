use std::collections::{BTreeMap, BTreeSet, VecDeque};

use machine_check_common::property::NextOperator;
use machine_check_common::{ExecError, StateId, ThreeValued};

use crate::model_check::property_checker::{CheckValue, LabellingComputer};
use crate::FullMachine;

impl<M: FullMachine> LabellingComputer<'_, M> {
    pub fn compute_next_labelling(
        &mut self,
        subproperty_index: usize,
        op: &NextOperator,
    ) -> Result<(), ExecError> {
        let ground_value = CheckValue::eigen(ThreeValued::from_bool(op.is_universal));
        self.compute_labelling(op.inner)?;

        // We need to compute states where the inner property was updated for their direct successors.

        let mut dirty = BTreeSet::new();

        let inner_computation = Self::computation(&self.updates, op.inner);

        //for state_id in self.space.states() {
        for state_id in inner_computation.iter().cloned() {
            for predecessor_id in self.space.direct_predecessor_iter(state_id.into()) {
                if let Ok(predecessor_id) = StateId::try_from(predecessor_id) {
                    dirty.insert(predecessor_id);
                }
            }
        }

        let mut update = BTreeMap::new();

        // For each state in dirty states, compute the new value from the successors.
        for dirty_id in dirty.iter().copied() {
            let old_successor = self
                .property_checker
                .get_labelling(subproperty_index)
                .get(&dirty_id)
                .and_then(|value| value.next_states.last().copied());

            let mut direct_successors = VecDeque::new();
            for successor_id in self.space.direct_successor_iter(dirty_id.into()) {
                let value = self
                    .property_checker
                    .get_labelling(op.inner)
                    .get(&successor_id)
                    .expect("Direct successor should have values")
                    .clone();
                if old_successor.is_some_and(|old_successor| old_successor == successor_id) {
                    direct_successors.push_front((successor_id, value));
                } else {
                    direct_successors.push_back((successor_id, value));
                }
            }

            let mut current_value = ground_value.clone();

            for (successor_id, successor_value) in &direct_successors {
                let new_valuation = if op.is_universal {
                    current_value.valuation & successor_value.valuation
                } else {
                    current_value.valuation | successor_value.valuation
                };

                if current_value.valuation != new_valuation {
                    current_value.valuation = new_valuation;
                    current_value.next_states = successor_value.next_states.clone();
                    current_value.next_states.push(*successor_id);
                }
            }

            update.insert(dirty_id, current_value);
        }

        let inner_computation = Self::computation_mut(&mut self.updates, op.inner);
        inner_computation.clear();

        self.update_subproperty(subproperty_index, update);

        Ok(())
    }
}
