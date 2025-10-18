use machine_check_common::{ExecError, StateId};

use crate::{
    model_check::property_checker::{
        labelling_cacher::LabellingCacher, value::TimedCheckValue, CheckChoice,
    },
    FullMachine,
};

impl<M: FullMachine> LabellingCacher<'_, M> {
    pub(super) fn compute_fixed_point_op(
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

        let mut timed = history.before_time(self.current_time, state_id);

        timed.value.choice = CheckChoice::FixedVariable(timed.time);

        Ok(timed)
    }
}
