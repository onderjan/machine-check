use std::collections::BTreeMap;
use std::fmt::Debug;

use machine_check_common::StateId;

use crate::model_check::property_checker::value::CheckValue;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FixedPointHistory {
    states: BTreeMap<StateId, CheckValue>,
}

impl FixedPointHistory {
    pub fn insert(&mut self, state_id: StateId, value: CheckValue) {
        log::trace!("Inserting state {} new value {:?}", state_id, value);
        self.states.insert(state_id, value);
    }

    pub fn require(&self, state_id: StateId) -> &CheckValue {
        self.get(state_id).expect("History value should be present")
    }

    pub fn get(&self, state_id: StateId) -> Option<&CheckValue> {
        self.states.get(&state_id)
    }

    /*pub fn remove_states(&mut self, removed_states: &BTreeSet<StateId>) {
        self.states
            .retain(|state_id, _| !removed_states.contains(state_id));
    }*/

    pub fn clear(&mut self) {
        self.states.clear();
    }
}
