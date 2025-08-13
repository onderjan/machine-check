use std::collections::BTreeSet;

use machine_check_common::property::NextOperator;
use machine_check_common::{ExecError, StateId};

use crate::model_check::property_checker::LabellingUpdater;
use crate::FullMachine;

impl<M: FullMachine> LabellingUpdater<'_, M> {
    pub(super) fn update_next_labelling(
        &mut self,
        op: &NextOperator,
    ) -> Result<BTreeSet<StateId>, ExecError> {
        let inner_updated = self.update_labelling(op.inner)?;

        let mut our_updated = BTreeSet::new();

        for state_id in inner_updated {
            for predecessor_id in self.space.direct_predecessor_iter(state_id.into()) {
                if let Ok(predecessor_id) = StateId::try_from(predecessor_id) {
                    our_updated.insert(predecessor_id);
                }
            }
        }

        Ok(our_updated)

        /*let mut retained_successors = BTreeSet::new();

        for state_id in our_updated.iter().copied() {
            retained_successors.extend(
                self.space
                    .direct_successor_iter(state_id.into())
                    .filter(|state_id| !inner_updated.contains_key(state_id)),
            );
        }

        let inner_retained = self
            .getter()
            .cache_labelling(op.inner, retained_successors.iter().copied())?;

        let mut successor_inner = inner_updated;
        successor_inner.extend(inner_retained);

        self.getter()
            .apply_next(op, our_updated.iter().copied(), successor_inner)*/
    }
}
