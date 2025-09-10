use std::ops::ControlFlow;

use log::trace;
use machine_check_common::ExecError;

use super::{select_history, select_history_mut};
use crate::{
    model_check::property_checker::labelling_updater::{
        fixed_point::{misc::intersect_state_set_and_map, FixedPointIterationParams},
        LabellingUpdater,
    },
    FullMachine,
};

impl<M: FullMachine> LabellingUpdater<'_, M> {
    pub(super) fn fixed_point_iteration(
        &mut self,
        params: &mut FixedPointIterationParams,
    ) -> Result<ControlFlow<(), ()>, ExecError> {
        let previous_time = self.current_time;
        // increment time
        self.current_time += 1;
        self.num_fixed_point_iterations += 1;

        // compute the iteration

        let mut current_update = self.update_labelling(params.inner_index)?;

        // states that have been updated in previous or current time also must be double-checked
        // only do this for backward-affected states since the others will not see a meaningful change during one iteration

        let history = select_history(&self.property_checker.histories, params.fixed_point_index);

        let affected_backward = self.property_checker.focus.affected_backward();

        // backward-affected states that have been updated in previous time are in danger since recomputation may be forced

        if let Some(history_previously_updated) = history.states_at_exact_time_opt(previous_time) {
            for (state_id, _value) in
                intersect_state_set_and_map(affected_backward, history_previously_updated)
            {
                let timed = self
                    .getter()
                    .compute_latest_timed(params.inner_index, state_id)?;

                current_update.insert(state_id, timed);
            }
        }
        // backward-affected states that have been updated in current time are in danger since there might be no current update

        if let Some(history_currently_updated) = history.states_at_exact_time_opt(self.current_time)
        {
            for (state_id, _value) in
                intersect_state_set_and_map(affected_backward, history_currently_updated)
            {
                let timed = self
                    .getter()
                    .compute_latest_timed(params.inner_index, state_id)?;

                current_update.insert(state_id, timed);
            }
        }

        // this also needs to be updated

        trace!(
            "Current update of fixed point {:?} in time {:?}: {:#?}",
            params.fixed_point_index,
            self.current_time,
            current_update
        );

        let history = select_history_mut(
            &mut self.property_checker.histories,
            params.fixed_point_index,
        );

        for (state_id, update_timed) in current_update {
            // check if the update differs
            // the timing of update is not relevant, as it will be the current time
            let update_value = update_timed.value;

            let now_timed = history.up_to_time(self.current_time, state_id);

            let is_dirty = self.property_checker.focus.dirty().contains(&state_id);

            if update_value == now_timed.value {
                // the values are exactly the same
                continue;
            }

            // the values are not exactly the same
            // but we do not want to update an unknown value
            if update_value.valuation() == now_timed.value.valuation()
                && now_timed.time != self.current_time
            {
                continue;
            }

            // make the state dirty, if it was not dirty, clear entries from current time
            if !is_dirty {
                self.property_checker
                    .focus
                    .insert_dirty(self.space, state_id);

                trace!(
                    "Inserted state {} to dirty, clearing entries from time {}",
                    state_id,
                    self.current_time
                );
                history.clear_entries_from(self.current_time, state_id);
            }

            // insert
            history.insert(self.current_time, state_id, update_value);
        }

        if !history.time_changes(self.current_time) {
            if let Some(old_computation_end_time) = params.old_computation_end_time {
                if self.current_time >= old_computation_end_time
                    || !history.range_changes(self.current_time, old_computation_end_time)
                {
                    return Ok(ControlFlow::Break(()));
                }
            } else {
                return Ok(ControlFlow::Break(()));
            }
        }

        Ok(ControlFlow::Continue(()))
    }
}
