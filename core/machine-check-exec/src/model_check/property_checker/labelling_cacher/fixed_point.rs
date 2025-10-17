use machine_check_common::{iir::ISubpropertyFixedPoint, ExecError, StateId};

use crate::{
    model_check::property_checker::{
        labelling_cacher::LabellingCacher,
        value::{CheckChoice, CheckValue, TimedCheckValue},
    },
    FullMachine,
};

impl<M: FullMachine> LabellingCacher<'_, M> {
    pub(super) fn compute_fixed_point_op(
        &self,
        op: &ISubpropertyFixedPoint,
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

        let timed = history.before_time(self.current_time, state_id);

        // the variable is the reason
        // include the timing of the value to precisely capture it
        Ok(TimedCheckValue {
            time: timed.time,
            value: CheckValue {
                valuation: timed.value.valuation,
                choice: CheckChoice::FixedVariable(timed.time),
            },
        })
    }
}
