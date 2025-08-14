use machine_check_common::{property::FixedPointOperator, ExecError, StateId};

use crate::{
    model_check::property_checker::{labelling_cacher::LabellingCacher, TimedCheckValue},
    FullMachine,
};

impl<M: FullMachine> LabellingCacher<'_, M> {
    pub(super) fn compute_fixed_point_op(
        &self,
        op: &FixedPointOperator,
        state_id: StateId,
    ) -> Result<TimedCheckValue, ExecError> {
        // the current valuation is equal to the inner valuation
        self.compute_latest_timed(op.inner, state_id)
    }

    pub fn compute_fixed_variable(
        &self,
        fixed_point_index: usize,
        state_id: StateId,
    ) -> Result<TimedCheckValue, ExecError> {
        // the fixed variables are handled by looking into the history
        let history = self
            .property_checker
            .histories
            .get(&fixed_point_index)
            .expect("History should exist for fixed point");

        let mut timed = history.before_time(self.current_time, state_id);
        // clear next as we are considering a new time instant
        timed.value.next_states.clear();
        Ok(timed)
    }
}
