use std::collections::BTreeSet;

use log::trace;
use machine_check_common::iir::IProperty;
use machine_check_common::StateId;

use crate::space::StateSpace;
use crate::FullMachine;

#[derive(Debug, Clone)]

pub struct Focus {
    dirty: BTreeSet<StateId>,
}

impl Focus {
    pub fn new(_property: &IProperty) -> Self {
        Self {
            dirty: BTreeSet::new(),
        }
    }

    pub fn clear(&mut self) {
        trace!("Cleared focus");
        self.dirty.clear();
    }

    /*pub fn regenerate<M: FullMachine>(&mut self, space: &StateSpace<M>, added: &BTreeSet<StateId>) {
        trace!("Regenerating, dirty before {:?}", self.dirty);
        let mut dirty = BTreeSet::new();
        std::mem::swap(&mut dirty, &mut self.dirty);
        self.clear();
        dirty.extend(added);

        for state_id in dirty {
            self.insert_dirty(space, state_id);
        }
    }

    pub fn remove_states(&mut self, removed_states: &BTreeSet<StateId>) {
        for state in removed_states {
            self.dirty.remove(state);
        }
    }*/

    pub fn dirty(&self) -> &BTreeSet<StateId> {
        &self.dirty
    }

    pub fn dirty_iter(&self) -> impl Iterator<Item = StateId> + use<'_> {
        self.dirty().iter().copied()
    }

    pub fn make_whole_dirty<M: FullMachine>(&mut self, space: &StateSpace<M>) {
        trace!("Making whole space dirty");
        for state_id in space.states() {
            self.insert_dirty(space, state_id);
        }
        trace!("Made whole space dirty");
    }

    pub fn insert_dirty<M: FullMachine>(&mut self, _space: &StateSpace<M>, state_id: StateId) {
        self.dirty.insert(state_id);
    }
}
