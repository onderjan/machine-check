use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Debug;

use machine_check_common::StateId;

use crate::model_check::property_checker::value::{CheckValue, TimedCheckValue};
use crate::model_check::property_checker::{squash_time, CheckChoice};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FixedPointHistory {
    times: BTreeMap<u64, BTreeMap<StateId, CheckValue>>,
    states: BTreeMap<StateId, BTreeMap<u64, CheckValue>>,
}

impl FixedPointHistory {
    pub fn clear_entries_from(&mut self, time_instant: u64, state_id: StateId) {
        let time_values = self.states.entry(state_id).or_default();
        log::trace!(
            "Clearing time values of state {}: {:?}",
            state_id,
            time_values
        );
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
            log::trace!("Removed entry of state {} at time {}", state_id, entry_time);

            let time_instant_state_map = self
                .times
                .get_mut(&entry_time)
                .expect("Entry time should have a map in times");
            time_instant_state_map.remove(&state_id);
            if time_instant_state_map.is_empty() {
                self.times.remove(&entry_time);
            }
        }
    }

    pub fn insert(&mut self, time_instant: u64, state_id: StateId, value: CheckValue) {
        if let Some(contained) = self.before_time_opt(time_instant, state_id) {
            if contained.value == value {
                // do not insert as it is already implied
                // but make sure there is nothing at this exact time
                if let Some(time_map) = self.times.get_mut(&time_instant) {
                    time_map.remove(&state_id);
                }

                if let Some(state_map) = self.states.get_mut(&state_id) {
                    state_map.remove(&time_instant);
                }

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

        log::trace!(
            "Inserted new value to state {} at time {}",
            state_id,
            time_instant
        );
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

    pub fn remove_states(&mut self, removed_states: &BTreeSet<StateId>) {
        self.states
            .retain(|state_id, _| !removed_states.contains(state_id));
        let mut times = BTreeMap::new();
        std::mem::swap(&mut times, &mut self.times);
        for (time, mut state_map) in times {
            state_map.retain(|state_id, _| !removed_states.contains(state_id));
            if !state_map.is_empty() {
                self.times.insert(time, state_map);
            }
        }
    }

    pub fn remove_times(&mut self, start: u64, end: u64) {
        let retain_fn = |time: &u64| *time < start || *time >= end;
        self.times.retain(|time, _| retain_fn(time));
        let mut states = BTreeMap::new();
        std::mem::swap(&mut states, &mut self.states);
        for (state, mut time_map) in states {
            time_map.retain(|time, _| retain_fn(time));
            if !time_map.is_empty() {
                self.states.insert(state, time_map);
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

    pub fn squash(&mut self, time_mapping: &BTreeMap<u64, u64>) {
        let mut original_times = BTreeMap::new();
        std::mem::swap(&mut original_times, &mut self.times);

        for (original_time, mut state_map) in original_times {
            for value in state_map.values_mut() {
                squash_value(value, time_mapping);
            }

            let squashed_time = squash_time(time_mapping, original_time);
            self.times.insert(squashed_time, state_map);
        }

        for time_map in self.states.values_mut() {
            let mut original_time_map = BTreeMap::new();
            std::mem::swap(&mut original_time_map, time_map);

            for (original_time, mut value) in original_time_map {
                let squashed_time = squash_time(time_mapping, original_time);
                squash_value(&mut value, time_mapping);

                time_map.insert(squashed_time, value);
            }
        }
    }

    pub fn time_keys(&self) -> impl Iterator<Item = u64> + use<'_> {
        self.times.keys().copied()
    }
}

fn squash_value(value: &mut CheckValue, time_mapping: &BTreeMap<u64, u64>) {
    if let CheckValue::Unknown(choices) = value {
        for choice in choices {
            if let CheckChoice::FixedVariable(time) = choice {
                *time = squash_time(time_mapping, *time);
            }
        }
    }
}
