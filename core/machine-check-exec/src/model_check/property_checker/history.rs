use std::collections::BTreeMap;
use std::fmt::Debug;

use log::trace;
use machine_check_common::{StateId, ThreeValued};

use crate::model_check::property_checker::squash_time;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct CheckValue {
    pub valuation: ThreeValued,
    pub next_states: Vec<StateId>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TimedCheckValue {
    pub time: u64,
    pub value: CheckValue,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FixedPointHistory {
    pub(super) times: BTreeMap<u64, BTreeMap<StateId, CheckValue>>,
    pub(super) states: BTreeMap<StateId, BTreeMap<u64, CheckValue>>,
}

impl FixedPointHistory {
    pub fn insert(&mut self, time_instant: u64, state_id: StateId, value: CheckValue) {
        trace!(
            "Inserting for state id {} and time instant {}: {:?}",
            state_id,
            time_instant,
            value
        );
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

        let Some((insert_time, check_value)) = history.range(0..time).last() else {
            panic!(
                "Last history of state {} before time {} should exist, but history is {:?}",
                state_id, time, self
            );
        };

        TimedCheckValue {
            time: *insert_time,
            value: check_value.clone(),
        }
    }

    pub fn clear(&mut self) {
        self.times.clear();
        self.states.clear();
    }

    pub fn time_changes(&self, time_instant: u64) -> bool {
        self.times.contains_key(&time_instant)
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
}

impl CheckValue {
    pub fn eigen(value: ThreeValued) -> Self {
        Self {
            valuation: value,
            next_states: vec![],
        }
    }
}

impl Debug for CheckValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {:?}", self.valuation, self.next_states)
    }
}
