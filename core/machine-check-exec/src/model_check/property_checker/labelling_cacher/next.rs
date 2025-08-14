use machine_check_common::property::NextOperator;
use machine_check_common::{ExecError, StateId, ThreeValued};

use crate::model_check::property_checker::labelling_cacher::LabellingCacher;
use crate::model_check::property_checker::{CheckValue, TimedCheckValue};
use crate::FullMachine;

impl<M: FullMachine> LabellingCacher<'_, M> {
    pub(super) fn compute_next_labelling(
        &self,
        op: &NextOperator,
        state_id: StateId,
    ) -> Result<TimedCheckValue, ExecError> {
        // cache inner labellings of successors
        for successor_id in self.space.direct_successor_iter(state_id.into()) {
            self.get_latest_timed(op.inner, successor_id)?;
        }

        self.apply_next(op, state_id)
    }

    pub fn apply_next(
        &self,
        op: &NextOperator,
        state_id: StateId,
    ) -> Result<TimedCheckValue, ExecError> {
        let ground_value = CheckValue::eigen(ThreeValued::from_bool(op.is_universal));

        // for speed, try to find the appropriate successor without sorting first
        // this can be no successor at all, if the ground value remains,
        // or a single successor with the appropriate valuation

        let mut current_valuation = ground_value.valuation;
        let mut found_successor = None;

        for successor_id in self.space.direct_successor_iter(state_id.into()) {
            let successor_timed = self.get_latest_timed(op.inner, successor_id)?;
            let successor_valuation = successor_timed.value.valuation;

            let is_better = if op.is_universal {
                successor_valuation < current_valuation
            } else {
                successor_valuation > current_valuation
            };

            if is_better {
                current_valuation = successor_valuation;
                found_successor = Some(successor_id);
            } else if successor_valuation == current_valuation {
                found_successor = None;
            }
        }

        if current_valuation == ground_value.valuation {
            // no successor
            return Ok(TimedCheckValue {
                time: 0,
                value: ground_value.clone(),
            });
        }

        if let Some(successor_id) = found_successor {
            // single allowed successor
            // add the successor id to next states to obtain our value
            let mut timed = self.get_latest_timed(op.inner, successor_id)?;
            timed.value.next_states.push(successor_id);
            return Ok(timed);
        };

        // we already know what valuation we have
        // we have to sort the successors that have the given valuation

        let mut successor_sorter = Vec::new();

        for successor_id in self.space.direct_successor_iter(state_id.into()) {
            let successor_timed = self.get_latest_timed(op.inner, successor_id)?;
            if successor_timed.value.valuation == current_valuation {
                successor_sorter.push((
                    (successor_timed.time, successor_id),
                    successor_timed.value.clone(),
                ));
            }
        }

        successor_sorter.sort_by(|(a_key, _), (b_key, _)| a_key.cmp(b_key));

        // the first successor is the wanted one

        let ((successor_time, successor_id), successor_value) = successor_sorter
            .first()
            .expect("There should be a first successor");

        // add the successor id to next states to obtain our value
        let mut timed = TimedCheckValue {
            time: *successor_time,
            value: successor_value.clone(),
        };

        assert!(!timed.value.next_states.contains(&state_id));
        assert!(!timed.value.next_states.contains(successor_id));

        timed.value.next_states.push(*successor_id);

        Ok(timed)
    }
}
