use std::collections::{BTreeMap, BTreeSet};

use machine_check_common::property::NextOperator;
use machine_check_common::{ExecError, StateId, ThreeValued};

use crate::model_check::property_checker::labelling_getter::LabellingGetter;
use crate::model_check::property_checker::{CheckValue, TimedCheckValue};
use crate::FullMachine;

impl<M: FullMachine> LabellingGetter<'_, M> {
    pub(super) fn get_next_labelling(
        &self,
        op: &NextOperator,
        states: &BTreeSet<StateId>,
    ) -> Result<BTreeMap<StateId, TimedCheckValue>, ExecError> {
        let mut successor_states = BTreeSet::new();

        for state_id in states.iter().copied() {
            successor_states.extend(self.space.direct_successor_iter(state_id.into()));
        }

        let successor_inner = self.get_labelling(op.inner, &successor_states)?;

        self.apply_next(op, states, successor_inner)
    }

    pub fn apply_next(
        &self,
        op: &NextOperator,
        states: &BTreeSet<StateId>,
        successor_inner: BTreeMap<StateId, TimedCheckValue>,
    ) -> Result<BTreeMap<StateId, TimedCheckValue>, ExecError> {
        let ground_value = CheckValue::eigen(ThreeValued::from_bool(op.is_universal));
        let mut result = BTreeMap::new();

        for state_id in states.iter().copied() {
            let successor_iter = self.space.direct_successor_iter(state_id.into());
            let mut sorted_successors = BTreeMap::new();

            for successor_id in successor_iter {
                let successor_timed = successor_inner
                    .get(&successor_id)
                    .expect("Successor should be in gotten state");
                sorted_successors.insert(
                    (successor_timed.time, successor_id),
                    successor_timed.value.clone(),
                );
            }

            let mut current_timed = TimedCheckValue {
                time: 0,
                value: ground_value.clone(),
            };

            for ((successor_time, successor_id), successor_value) in sorted_successors {
                let new_valuation = if op.is_universal {
                    current_timed.value.valuation & successor_value.valuation
                } else {
                    current_timed.value.valuation | successor_value.valuation
                };

                if current_timed.value.valuation != new_valuation {
                    current_timed.value.valuation = new_valuation;
                    current_timed.value.next_states = successor_value.next_states.clone();
                    current_timed.value.next_states.push(successor_id);
                    current_timed.time = successor_time;
                }
            }
            result.insert(state_id, current_timed);
        }
        Ok(result)
    }
}
