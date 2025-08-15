use std::collections::BTreeMap;

use machine_check_common::property::BiLogicOperator;
use machine_check_common::{ExecError, StateId};

use crate::model_check::property_checker::history::TimedCheckValue;
use crate::model_check::property_checker::labelling_updater::LabellingUpdater;
use crate::model_check::property_checker::{BiChoice, LabellingCacher};
use crate::FullMachine;

impl<M: FullMachine> LabellingUpdater<'_, M> {
    pub(super) fn update_negation(
        &mut self,
        inner: usize,
    ) -> Result<BTreeMap<StateId, TimedCheckValue>, ExecError> {
        let mut result = self.update_labelling(inner)?;
        for timed in result.values_mut() {
            timed.value.valuation = !timed.value.valuation;
        }

        Ok(result)
    }

    pub(super) fn update_binary_op(
        &mut self,
        op: &BiLogicOperator,
    ) -> Result<BTreeMap<StateId, TimedCheckValue>, ExecError> {
        let mut result = self.update_labelling(op.a)?;
        let mut result_b = self.update_labelling(op.b)?;

        for (state_id, timed) in result.iter_mut() {
            let timed_b = if let Some(timed_b) = result_b.remove(state_id) {
                timed_b
            } else {
                self.getter().compute_latest_timed(op.b, *state_id)?
            };

            if matches!(
                LabellingCacher::<M>::choose_binary_op(op, timed, &timed_b),
                BiChoice::Right
            ) {
                *timed = timed_b;
            };
        }

        for (state_id, timed_b) in result_b {
            let timed_a = self.getter().compute_latest_timed(op.a, state_id)?;

            let timed_result = match LabellingCacher::<M>::choose_binary_op(op, &timed_a, &timed_b)
            {
                BiChoice::Left => timed_a,
                BiChoice::Right => timed_b,
            };

            result.insert(state_id, timed_result);
        }

        Ok(result)
    }
}
