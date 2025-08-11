use machine_check_common::property::BiLogicOperator;
use machine_check_common::ExecError;

use crate::model_check::property_checker::labelling_updater::LabellingUpdater;
use crate::FullMachine;

impl<M: FullMachine> LabellingUpdater<'_, M> {
    pub(super) fn compute_negation(&mut self, inner: usize) -> Result<(), ExecError> {
        self.compute_labelling(inner)?;

        // TODO updates

        Ok(())
    }

    pub(super) fn compute_binary_op(&mut self, op: &BiLogicOperator) -> Result<(), ExecError> {
        self.compute_labelling(op.a)?;
        self.compute_labelling(op.b)?;

        // TODO updates

        Ok(())

        /*let mut result_a = self.compute_labelling(op.a)?;
        let mut result_b = self.compute_labelling(op.b)?;

        self.complete_labelling(op.a, &mut result_a, &result_b)?;
        self.complete_labelling(op.b, &mut result_b, &result_a)?;

        let mut result = BTreeMap::new();
        for (state_id, a_timed) in result_a {
            let b_timed = result_b
                .remove(&state_id)
                .expect("Binary operation labelling should be completed");

            let result_value = LabellingGetter::<M>::apply_binary_op(op, a_timed, b_timed);
            result.insert(state_id, result_value);
        }

        Ok(result)*/
    }

    /*
    fn complete_labelling(
        &self,
        our_index: usize,
        our_result: &mut BTreeMap<StateId, TimedCheckValue>,
        other_result: &BTreeMap<StateId, TimedCheckValue>,
    ) -> Result<(), ExecError> {
        let mut fetch = BTreeSet::new();
        for state_id in other_result.keys().copied() {
            if !our_result.contains_key(&state_id) {
                fetch.insert(state_id);
            }
        }
        our_result.extend(
            self.getter()
                .cache_labelling(our_index, fetch.iter().copied())?,
        );
        Ok(())
    }*/
}
