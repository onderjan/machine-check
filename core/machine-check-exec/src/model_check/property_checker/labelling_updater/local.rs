use std::collections::{BTreeMap, BTreeSet};

use log::trace;

use machine_check_common::iir::ISubpropertyFunc;
use machine_check_common::{ExecError, StateId};

use crate::model_check::property_checker::labelling_updater::LabellingUpdater;
use crate::model_check::property_checker::value::TimedCheckValue;
use crate::FullMachine;

impl<M: FullMachine> LabellingUpdater<'_, M> {
    pub(super) fn update_func(
        &mut self,
        op: &ISubpropertyFunc,
    ) -> Result<BTreeMap<StateId, TimedCheckValue>, ExecError> {
        trace!("Updating function labelling");

        let mut labellings = BTreeMap::new();
        let mut updated_states = BTreeSet::new();

        for dependency in &op.dependencies {
            let labelling = if op.children.contains(dependency) {
                self.update_labelling(*dependency)?
            } else {
                let mut labelling = BTreeMap::new();
                for state_id in self.property_checker.focus.dirty_iter() {
                    labelling.insert(
                        state_id,
                        self.getter().compute_latest_timed(*dependency, state_id)?,
                    );
                }
                labelling
            };

            updated_states.extend(labelling.keys());
            labellings.insert(*dependency, labelling);
        }

        let mut result = BTreeMap::new();

        for state_id in updated_states {
            let value = self.getter().apply_func(op, state_id, &mut labellings)?;
            result.insert(state_id, value);
        }

        Ok(result)
    }
}
