use std::ops::ControlFlow;

use log::trace;
use machine_check_common::ExecError;

use super::select_history_mut;
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
        self.num_fixed_point_iterations += 1;

        // compute the iteration

        let current_update = self.update_labelling(params.inner_index)?;

        // this also needs to be updated

        trace!(
            "Current update of fixed point {:?}: {:#?}",
            params.fixed_point_index,
            current_update
        );

        let history = select_history_mut(
            &mut self.property_checker.histories,
            params.fixed_point_index,
        );

        let mut control_flow = ControlFlow::Break(());

        for (state_id, update_value) in current_update {
            // check if the update differs

            let now_value = history.require(state_id);
            if update_value.valuation == now_value.valuation {
                continue;
            }

            // insert
            history.insert(state_id, update_value);
            control_flow = ControlFlow::Continue(());
        }

        Ok(control_flow)
    }
}
