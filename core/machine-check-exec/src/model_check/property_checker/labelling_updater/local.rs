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

        for dependency_index in &op.dependencies {
            let labelling =
                self.update_labelling(*dependency_index, op.children.contains(dependency_index))?;
            updated_states.extend(labelling.keys());
            labellings.insert(*dependency_index, labelling);
        }

        let mut result = BTreeMap::new();

        for state_id in updated_states {
            let value = self.getter().apply_func(op, state_id, &mut labellings)?;
            result.insert(state_id, value);
        }

        Ok(result)
    }
}
