use std::{collections::BTreeMap, ops::ControlFlow};

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

        let mut updated = self.update_labelling(params.inner_index)?;

        // add previously updated

        let history = select_history(&self.property_checker.histories, params.fixed_point_index);
        if let Some(previously_updated) = history.states_at_exact_time_opt(self.current_time) {
            // this also needs to be updated
            for state_id in previously_updated.keys().copied() {
                if self.space.contains_state(state_id) {
                    updated.insert(state_id);
                }
            }
        }

        // TODO: compute the current update properly
        let mut current_update = BTreeMap::new();
        for state_id in updated {
            let value = self
                .getter()
                .get_latest_timed(params.inner_index, state_id)?
                .value;
            current_update.insert(state_id, value);
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

        for (state_id, update_value) in current_update {
            // check if the update differs

            let now_timed = history.up_to_time(self.current_time, state_id);

            if update_value == now_timed.value {
                continue;
            }

            /*trace!(
                "Inserting dirty state {}: now {:?}, update: {:?}",
                state_id,
                now_timed,
                update_value
            );*/

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
