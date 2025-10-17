use std::collections::BTreeMap;

use machine_check_common::iir::ISubpropertyFunc;
use machine_check_common::{ExecError, StateId};

use crate::model_check::property_checker::labelling_updater::LabellingUpdater;
use crate::model_check::property_checker::value::TimedCheckValue;
use crate::FullMachine;

impl<M: FullMachine> LabellingUpdater<'_, M> {
    pub(super) fn update_func(
        &self,
        op: &ISubpropertyFunc,
    ) -> Result<BTreeMap<StateId, TimedCheckValue>, ExecError> {
        todo!()
    }

    /*pub(super) fn update_negation(
        &mut self,
        inner: usize,
    ) -> Result<BTreeMap<StateId, TimedCheckValue>, ExecError> {
        let inner_result = self.update_labelling(inner)?;
        let mut result = BTreeMap::new();

        for (state_id, timed) in inner_result {
            let timed = LabellingCacher::<M>::apply_negation(timed);

            result.insert(state_id, timed);
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

            let choice = LabellingCacher::<M>::choose_binary_op(op, timed, &timed_b);

            if matches!(choice, BiChoice::Right) {
                *timed = timed_b;
            };

            // add the choice
            if let CheckValue::Unknown(choices) = &mut timed.value {
                choices.push(CheckChoice::BiLogic(choice));
            };
        }

        for (state_id, timed_b) in result_b {
            let timed_a = self.getter().compute_latest_timed(op.a, state_id)?;

            let choice = LabellingCacher::<M>::choose_binary_op(op, &timed_a, &timed_b);
            let mut timed = match choice {
                BiChoice::Left => timed_a,
                BiChoice::Right => timed_b,
            };

            // add the choice
            if let CheckValue::Unknown(choices) = &mut timed.value {
                choices.push(CheckChoice::BiLogic(choice));
            };

            result.insert(state_id, timed);
        }

        Ok(result)
    }*/
}
