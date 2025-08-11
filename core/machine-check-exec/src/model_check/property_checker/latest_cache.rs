use std::collections::BTreeMap;

use machine_check_common::StateId;

use crate::model_check::property_checker::history::TimedCheckValue;

#[derive(Debug, Clone, Default)]
struct SubpropertyCache {
    values: BTreeMap<StateId, TimedCheckValue>,
}

#[derive(Debug, Clone)]
pub struct LatestCache {
    subproperty_map: BTreeMap<usize, SubpropertyCache>,
}

impl LatestCache {
    pub fn new() -> Self {
        Self {
            subproperty_map: BTreeMap::new(),
        }
    }

    pub fn get(&self, subproperty_index: usize, state_id: StateId) -> Option<TimedCheckValue> {
        let state_map = self.subproperty_map.get(&subproperty_index)?;
        state_map.values.get(&state_id).cloned()
    }

    pub fn insert(&mut self, subproperty_index: usize, state_id: StateId, timed: TimedCheckValue) {
        let state_map = self.subproperty_map.entry(subproperty_index).or_default();
        state_map.values.insert(state_id, timed);
    }

    pub fn clear_all(&mut self) {
        self.subproperty_map.clear();
    }
}
