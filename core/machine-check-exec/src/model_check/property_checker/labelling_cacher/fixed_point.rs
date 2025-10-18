use machine_check_common::{ExecError, StateId};

use crate::{
    model_check::property_checker::{
        labelling_cacher::LabellingCacher, value::CheckValue, CheckChoice,
    },
    FullMachine,
};

impl<M: FullMachine> LabellingCacher<'_, M> {
    pub(super) fn compute_fixed_point_op(
        &self,
        fixed_point_index: usize,
        state_id: StateId,
    ) -> Result<CheckValue, ExecError> {
        // look into the history
        let history = self
            .property_checker
            .histories
            .get(&fixed_point_index)
            .expect("History should exist for fixed point");

        let timed = history.require(state_id);

        Ok(CheckValue {
            valuation: timed.valuation,
            choice: CheckChoice::FixedPoint,
        })
    }
}
