use std::ops::ControlFlow;

use log::trace;
use machine_check_common::ExecError;

use super::{select_history, select_history_mut};
use crate::{
    model_check::property_checker::labelling_updater::{
        fixed_point::FixedPointIterationParams, LabellingUpdater,
    },
    FullMachine,
};

impl<M: FullMachine> LabellingUpdater<'_, M> {
    pub(super) fn fixed_point_iteration(
        &mut self,
        params: &mut FixedPointIterationParams,
    ) -> Result<ControlFlow<(), ()>, ExecError> {
        // increment time
        self.current_time += 1;
        self.num_fixed_point_iterations += 1;

        // compute the iteration

        let mut current_update = self.update_labelling(params.inner_index)?;

        // add previously updated
        // TODO: only do this for the states that really need this (affected backward?)

        let history = select_history(&self.property_checker.histories, params.fixed_point_index);
        if let Some(previously_updated) = history.states_at_exact_time_opt(self.current_time) {
            // this also needs to be updated
            for state_id in previously_updated.keys().copied() {
                let affected = self
                    .property_checker
                    .focus
                    .affected_backward()
                    .contains(&state_id);

                if affected && self.space.contains_state(state_id) {
                    let timed = self
                        .getter()
                        .compute_latest_timed(params.inner_index, state_id)?;

                    current_update.insert(state_id, timed);
                }
            }
        }

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

            if update_value == now_timed.value {
                continue;
            }

            // the update differs
            // insert the state and make it dirty

            history.insert(self.current_time, state_id, update_value);
            self.property_checker
                .focus
                .insert_dirty(self.space, state_id);
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
