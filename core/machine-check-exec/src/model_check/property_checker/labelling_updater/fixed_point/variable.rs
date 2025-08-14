use std::collections::BTreeMap;

use machine_check_common::{ExecError, StateId};

use super::select_history;
use crate::model_check::property_checker::history::TimedCheckValue;
use crate::model_check::property_checker::LabellingUpdater;
use crate::FullMachine;

impl<M: FullMachine> LabellingUpdater<'_, M> {
    pub(in super::super) fn update_fixed_variable(
        &mut self,
        fixed_point_index: usize,
    ) -> Result<BTreeMap<StateId, TimedCheckValue>, ExecError> {
        // update the values of the states that are dirty or can affect dirty and have been changed in last time instant
        let mut update = BTreeMap::new();
        let affected_forward = self.property_checker.focus.affected_forward();
        let history = select_history(&self.property_checker.histories, fixed_point_index);
        let last_time = self
            .current_time
            .checked_sub(1)
            .expect("Updates should not commence at time zero");
        let Some(changed_states) = history.states_at_exact_time_opt(last_time) else {
            return Ok(BTreeMap::new());
        };

        // iterate over the smaller collection to intersect it
        if affected_forward.len() <= changed_states.len() {
            for state_id in affected_forward {
                if let Some(changed_value) = changed_states.get(state_id) {
                    let mut update_value = changed_value.clone();
                    update_value.next_states.clear();
                    update.insert(*state_id, TimedCheckValue::new(last_time, update_value));
                }
            }
        } else {
            for (state_id, changed_value) in changed_states {
                if affected_forward.contains(state_id) {
                    let mut update_value = changed_value.clone();
                    update_value.next_states.clear();
                    update.insert(*state_id, TimedCheckValue::new(last_time, update_value));
                }
            }
        }

        Ok(update)
    }
}
