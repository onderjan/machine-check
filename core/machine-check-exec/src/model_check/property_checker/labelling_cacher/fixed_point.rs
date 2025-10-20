use machine_check_common::{iir::property::ISubpropertyFixedPoint, ExecError, StateId};

use crate::{
    model_check::property_checker::{
        labelling_cacher::LabellingCacher, value::TimedCheckValue, CheckChoice, CheckValue,
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
        let inner_timed = self.compute_latest_timed(op.inner, state_id)?;

        let value = if let CheckValue::Unknown(choice) = inner_timed.value {
            CheckValue::Unknown(Box::new(CheckChoice::FixedPoint(choice)))
        } else {
            inner_timed.value
        };

        Ok(TimedCheckValue {
            time: inner_timed.time,
            value,
        })
    }

    pub fn compute_fixed_variable(
        &self,
        fixed_point_index: usize,
        state_id: StateId,
    ) -> Result<TimedCheckValue, ExecError> {
        // look into the history
        let history = self
            .property_checker
            .histories
            .get(&fixed_point_index)
            .expect("History should exist for fixed point");

        let inner_timed = history.before_time(self.current_time, state_id);

        let value = if let CheckValue::Unknown(_) = inner_timed.value {
            CheckValue::Unknown(Box::new(CheckChoice::FixedVariable(inner_timed.time)))
        } else {
            inner_timed.value
        };

        Ok(TimedCheckValue {
            time: inner_timed.time,
            value,
        })
    }
}
