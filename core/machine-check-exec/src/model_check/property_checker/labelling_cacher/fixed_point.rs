use machine_check_common::{property::FixedPointOperator, ExecError, StateId};

use crate::{
    model_check::property_checker::{
        labelling_cacher::LabellingCacher,
        value::{CheckValue, Reason, TimedCheckValue},
    },
    FullMachine,
};

impl<M: FullMachine> LabellingCacher<'_, M> {
    pub(super) fn compute_fixed_point_op(
        &self,
        op: &FixedPointOperator,
        state_id: StateId,
    ) -> Result<TimedCheckValue, ExecError> {
        // the current valuation is equal to the inner valuation
        let mut timed = self.compute_latest_timed(op.inner, state_id)?;
        // add the reason
        if let CheckValue::Unknown(reasons) = &mut timed.value {
            reasons.push(Reason::FixedPoint);
        };
        Ok(timed)
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
        if let CheckValue::Unknown(reasons) = &mut timed.value {
            // clear the reasons and add the variable as the only reason
            reasons.clear();
            reasons.push(Reason::FixedVariable);
        };

        Ok(timed)
    }
}
