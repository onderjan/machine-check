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
        // increment time
        self.current_time += 1;
        self.num_fixed_point_iterations += 1;

        // compute the iteration

        let mut current_update = self.update_labelling(params.inner_index)?;

        // add previously updated
        // only do this for backward-affected states since the others will not see a meaningful change during one iteration

        let history = select_history(&self.property_checker.histories, params.fixed_point_index);
        let affected_backward = self.property_checker.focus.affected_backward();
        if let Some(previously_updated) = history.states_at_exact_time_opt(self.current_time) {
            // this also needs to be updated
            // iterate over the smaller collection
            // note that this is not easily

            for (state_id, _value) in
                intersect_state_set_and_map(affected_backward, previously_updated)
            {
                let timed = self
                    .getter()
                    .compute_latest_timed(params.inner_index, state_id)?;

                current_update.insert(state_id, timed);
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
