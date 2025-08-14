use std::collections::{BTreeMap, BTreeSet};

use machine_check_common::property::NextOperator;
use machine_check_common::{ExecError, StateId};

use crate::model_check::property_checker::history::TimedCheckValue;
use crate::model_check::property_checker::LabellingUpdater;
use crate::FullMachine;

impl<M: FullMachine> LabellingUpdater<'_, M> {
    pub(super) fn update_next_labelling(
        &mut self,
        op: &NextOperator,
    ) -> Result<BTreeMap<StateId, TimedCheckValue>, ExecError> {
        let inner_updated = self.update_labelling(op.inner)?;

        let mut result = BTreeMap::new();

        let states = BTreeSet::from_iter(inner_updated.keys().copied());

        for state_id in states {
            for predecessor_id in self.space.direct_predecessor_iter(state_id.into()) {
                let Ok(predecessor_id) = StateId::try_from(predecessor_id) else {
                    // skip root node
                    continue;
                };

                let predecessor_value =
                    self.getter()
                        .apply_next(op, predecessor_id, &mut BTreeMap::new())?;
                result.insert(predecessor_id, predecessor_value);
            }
        }

        Ok(result)
    }
}
