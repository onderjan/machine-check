use machine_check_common::{property::FixedPointOperator, ExecError, StateId};

use crate::{
    model_check::property_checker::{labelling_getter::LabellingGetter, TimedCheckValue},
    FullMachine,
};

impl<M: FullMachine> LabellingGetter<'_, M> {
    pub(super) fn cache_fixed_point_op(
        &self,
        op: &FixedPointOperator,
        state_id: StateId,
    ) -> Result<TimedCheckValue, ExecError> {
        // the current valuation is equal to the inner valuation
        self.cache_labelling(op.inner, state_id)?;
        Ok(self.property_checker.get_cached(op.inner, state_id))
    }

    pub fn cache_fixed_variable(
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

        let timed = history.before_time(self.current_time, state_id);
        Ok(timed)
    }
}
