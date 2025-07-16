mod labelling_computer;

use std::collections::{BTreeMap, BTreeSet};

use log::{log_enabled, trace};
use machine_check_common::{check::Property, ExecError, StateId, ThreeValued};
use mck::concr::FullMachine;

use crate::{
    model_check::{
        history::{HistoryIndex, Label},
        property_checker::labelling_computer::LabellingComputer,
    },
    space::StateSpace,
};

#[derive(Debug)]
pub struct PropertyChecker {
    check_map: BTreeMap<usize, CheckInfo>,
    very_dirty: BTreeSet<StateId>,
}

#[derive(Debug)]
struct CheckInfo {
    pub labelling: BTreeMap<StateId, Label>,
    pub dirty: BTreeSet<StateId>,
    pub fixed_reaches: BTreeSet<HistoryIndex>,
}

impl PropertyChecker {
    pub fn new() -> Self {
        Self {
            check_map: BTreeMap::new(),
            very_dirty: BTreeSet::new(),
        }
    }

    pub fn purge_states(&mut self, purge_states: &BTreeSet<StateId>) {
        self.very_dirty.extend(purge_states.iter());

        for check_info in self.check_map.values_mut() {
            check_info.dirty.extend(purge_states.iter());

            for state_id in purge_states {
                check_info.labelling.remove(state_id);
            }
        }
    }

    pub fn compute_interpretation<M: FullMachine>(
        &mut self,
        space: &StateSpace<M>,
        property: &Property,
    ) -> Result<ThreeValued, ExecError> {
        let mut labelling_computer = LabellingComputer::new(self, property, space);
        labelling_computer.compute()?;

        self.very_dirty.clear();
        let labelling = self.get_labelling(0);
        // conventionally, the property must hold in all initial states
        let mut result = ThreeValued::True;
        for initial_state_id in space.initial_iter() {
            let label = labelling
                .get(&initial_state_id)
                .expect("Labelling should contain initial state");
            let state_value = label.last_point().value;

            result = result & state_value;
        }

        if log_enabled!(log::Level::Trace) {
            trace!("Computed interpretation of {:?}", property);

            for (subproperty_index, check_info) in &self.check_map {
                let subproperty = property.subproperty_entry(*subproperty_index);

                let mut display = format!(
                    "Subproperty {} ({:?}): resets {:?}, labelling [\n",
                    subproperty_index, subproperty, check_info.fixed_reaches
                );
                for (state_id, label) in &check_info.labelling {
                    display.push_str(&format!("\t{}: {:?}\n", state_id, label));
                }
                display.push_str("]\n");

                trace!("{}", display);
            }
        }

        Ok(result)
    }

    pub fn get_state_label(&self, subproperty_index: usize, state_index: StateId) -> &Label {
        // TODO: this is wasteful when looking at multiple states
        self.get_labelling(subproperty_index)
            .get(&state_index)
            .expect("Should contain state labelling")
    }

    pub fn get_state_root_label(&self, state_index: StateId) -> &Label {
        self.get_state_label(0, state_index)
    }

    pub fn get_labelling(&self, subproperty_index: usize) -> &BTreeMap<StateId, Label> {
        &self
            .check_map
            .get(&subproperty_index)
            .expect("Labelling should be present")
            .labelling
    }
}
