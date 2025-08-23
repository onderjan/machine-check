use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Debug;

use machine_check_common::{ParamValuation, StateId};

use crate::model_check::property_checker::squash_time;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct CheckValue {
    pub valuation: ParamValuation,
    pub next_states: Vec<StateId>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TimedCheckValue {
    pub time: u64,
    pub value: CheckValue,
}

impl TimedCheckValue {
    pub fn new(time: u64, value: CheckValue) -> Self {
        TimedCheckValue { time, value }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FixedPointHistory {
    times: BTreeMap<u64, BTreeMap<StateId, CheckValue>>,
    states: BTreeMap<StateId, BTreeMap<u64, CheckValue>>,
}

impl FixedPointHistory {
    pub fn insert(&mut self, time_instant: u64, state_id: StateId, value: CheckValue) {
        let time_values = self.states.entry(state_id).or_default();

        // clear the entries at or after this time for this state
        loop {
            let Some((entry_time, _)) = time_values.last_key_value() else {
                // nothing in the map, break
                break;
            };

            let entry_time = *entry_time;

            if entry_time < time_instant {
                // entry time is smaller than insert time instant
                // since it is last, every entry time will be smaller
                // so we can break
                break;
            }

            // remove entry both in time and state maps
            time_values.remove(&entry_time);

            let time_instant_state_map = self
                .times
                .get_mut(&entry_time)
                .expect("Entry time should have a map in times");
            time_instant_state_map.remove(&state_id);
            if time_instant_state_map.is_empty() {
                self.times.remove(&entry_time);
            }
        }

        if let Some(contained) = self.before_time_opt(time_instant, state_id) {
            if contained.value == value {
                // do not insert as it is already implied
                return;
            }
        }

        // insert the new entry
        self.times
            .entry(time_instant)
            .or_default()
            .insert(state_id, value.clone());

        self.states
            .entry(state_id)
            .or_default()
            .insert(time_instant, value);
    }

    pub fn before_time(&self, time: u64, state_id: StateId) -> TimedCheckValue {
        let Some(history) = self.states.get(&state_id) else {
            panic!(
                "State {} should have history when querying before time {}, but history is {:?}",
                state_id, time, self
            );
        };

        let Some((insert_time, check_value)) = history.range(0..time).next_back() else {
            panic!(
                "Last history of state {} before time {} should exist, but history is {:?}",
                state_id, time, self
            );
        };

        TimedCheckValue::new(*insert_time, check_value.clone())
    }

    pub fn up_to_time(&self, time: u64, state_id: StateId) -> TimedCheckValue {
        self.before_time(time + 1, state_id)
    }

    pub fn before_time_opt(&self, time: u64, state_id: StateId) -> Option<TimedCheckValue> {
        let history = self.states.get(&state_id)?;
        let (insert_time, check_value) = history.range(0..time).next_back()?;

        Some(TimedCheckValue::new(*insert_time, check_value.clone()))
    }

    pub fn states_at_exact_time_opt(&self, time: u64) -> Option<&BTreeMap<StateId, CheckValue>> {
        self.times.get(&time)
    }

    pub fn retain_states(&mut self, states: &BTreeSet<StateId>) {
        self.states.retain(|state_id, _| states.contains(state_id));
        let mut times = BTreeMap::new();
        std::mem::swap(&mut times, &mut self.times);
        for (time, mut state_map) in times {
            state_map.retain(|state_id, _| states.contains(state_id));
            if !state_map.is_empty() {
                self.times.insert(time, state_map);
            }
        }
    }

    pub fn clear(&mut self) {
        self.times.clear();
        self.states.clear();
    }

    pub fn time_changes(&self, time_instant: u64) -> bool {
        self.times.contains_key(&time_instant)
    }

    pub fn range_changes(&self, start: u64, end: u64) -> bool {
        self.times.range(start..end).next().is_some()
    }

    pub fn for_state(&self, state_id: StateId) -> Option<&BTreeMap<u64, CheckValue>> {
        self.states.get(&state_id)
    }

    pub fn squash(&mut self, time_subtracts: &BTreeMap<u64, u64>, after_last_time: u64) {
        let mut original_times = BTreeMap::new();
        std::mem::swap(&mut original_times, &mut self.times);

        for (original_time, state_map) in original_times {
            let squashed_time = squash_time(time_subtracts, after_last_time, original_time);
            self.times.insert(squashed_time, state_map);
        }

        for time_map in self.states.values_mut() {
            let mut original_time_map = BTreeMap::new();
            std::mem::swap(&mut original_time_map, time_map);

            for (original_time, value) in original_time_map {
                let squashed_time = squash_time(time_subtracts, after_last_time, original_time);

                time_map.insert(squashed_time, value);
            }
        }
    }

    pub fn time_keys(&self) -> impl Iterator<Item = u64> + use<'_> {
        self.times.keys().copied()
    }
}

impl CheckValue {
    pub fn eigen(value: ParamValuation) -> Self {
        Self {
            valuation: value,
            next_states: vec![],
        }
    }
}

impl Debug for CheckValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} {:?}", self.valuation, self.next_states)
    }
}
