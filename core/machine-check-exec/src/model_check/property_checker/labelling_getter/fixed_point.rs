use std::collections::{BTreeMap, BTreeSet};

use machine_check_common::{property::FixedPointOperator, ExecError, StateId};

use crate::{
    model_check::property_checker::{labelling_getter::LabellingGetter, TimedCheckValue},
    FullMachine,
};

impl<M: FullMachine> LabellingGetter<'_, M> {
    pub(super) fn get_fixed_point_op(
        &self,
        op: &FixedPointOperator,
        states: &BTreeSet<StateId>,
    ) -> Result<BTreeMap<StateId, TimedCheckValue>, ExecError> {
        // we just get the current valuation, which is the inner valuation
        self.get_labelling(op.inner, states)
    }

    pub fn get_fixed_variable(
        &self,
        fixed_point_index: usize,
        states: &BTreeSet<StateId>,
    ) -> Result<BTreeMap<StateId, TimedCheckValue>, ExecError> {
        let mut update = BTreeMap::new();

        let history = self
            .property_checker
            .histories
            .get(&fixed_point_index)
            .expect("History should exist for fixed point");

        for state_id in states {
            let timed = history.before_time(self.current_time, *state_id);
            update.insert(*state_id, timed);
        }

        Ok(update)
    }
}
