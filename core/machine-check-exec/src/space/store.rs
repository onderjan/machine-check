use std::{collections::BTreeSet, num::NonZeroU64};

use bimap::{BiMap, Overwritten};
use machine_check_common::StateId;

use crate::{AbstrPanicState, WrappedState};

use mck::{concr::FullMachine, misc::MetaWrap};
use std::fmt::Debug;

pub struct StateStore<M: FullMachine> {
    /// Bidirectional map from state ids to the state values.
    map: BiMap<StateId, WrappedState<M>>,
    /// Next state id.
    next_state_id: StateId,
}

impl<M: FullMachine> Debug for StateStore<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StateStore")
            .field("map", &self.map)
            .field("next_state_id", &self.next_state_id)
            .finish()
    }
}

impl<M: FullMachine> StateStore<M> {
    pub fn new() -> Self {
        Self {
            map: BiMap::new(),
            next_state_id: StateId(NonZeroU64::MIN),
        }
    }

    // Get state id, add if necessary.
    pub fn state_id(&mut self, state: AbstrPanicState<M>) -> StateId {
        // Check if the state already corresponds to some id.
        let state = MetaWrap(state);
        if let Some(state_id) = self.map.get_by_right(&state) {
            // return that we have not inserted and the id
            return *state_id;
        };

        // Add state to the map with the next state id.
        let inserted_state_id = self.next_state_id;
        assert!(matches!(
            self.map.insert(inserted_state_id, state),
            Overwritten::Neither
        ));

        // Increment the next state id
        match self.next_state_id.0.checked_add(1) {
            Some(result) => self.next_state_id.0 = result,
            None => {
                // should never reasonably happen
                panic!("Next state id counter should not overflow");
            }
        };

        // Return the id.
        inserted_state_id
    }

    pub fn state_data(&self, state_id: StateId) -> &AbstrPanicState<M> {
        &self
            .map
            .get_by_left(&state_id)
            .expect("State should be in state map")
            .0
    }

    pub fn state_id_iter(&self) -> impl Iterator<Item = StateId> + '_ {
        self.map.left_values().cloned()
    }

    pub fn state_iter(&self) -> impl Iterator<Item = (StateId, &AbstrPanicState<M>)> + '_ {
        self.map
            .iter()
            .map(|(state_id, state_data)| (*state_id, &state_data.0))
    }

    pub fn num_states(&self) -> usize {
        self.map.len()
    }

    pub fn retain_states(&mut self, states: &BTreeSet<StateId>) {
        self.map
            .retain(|state_id, _state_data| states.contains(state_id));
    }
}
