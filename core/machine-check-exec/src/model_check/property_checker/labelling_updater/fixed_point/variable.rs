use std::collections::BTreeSet;

use machine_check_common::{ExecError, StateId};

use super::select_history;
use crate::model_check::property_checker::LabellingUpdater;
use crate::FullMachine;

impl<M: FullMachine> LabellingUpdater<'_, M> {
    pub fn update_fixed_variable(
        &mut self,
        fixed_point_index: usize,
    ) -> Result<BTreeSet<StateId>, ExecError> {
        // update the values of the states that are dirty or can affect dirty and have been changed in last time instant
        let mut update = BTreeSet::new();
        let affected_forward = self.property_checker.focus.affected_forward();
        let history = select_history(&self.property_checker.histories, fixed_point_index);
        let Some(changed_states) = history.states_at_exact_time_opt(self.current_time - 1) else {
            return Ok(BTreeSet::new());
        };

        // iterate over the smaller collection to intersect it
        if affected_forward.len() <= changed_states.len() {
            for state_id in affected_forward {
                if changed_states.contains_key(state_id) {
                    update.insert(*state_id);
                }
            }
        } else {
            for state_id in changed_states.keys() {
                if affected_forward.contains(state_id) {
                    update.insert(*state_id);
                }
            }
        }

        Ok(update)
    }
}
