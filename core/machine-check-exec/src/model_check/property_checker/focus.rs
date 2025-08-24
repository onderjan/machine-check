use std::collections::BTreeSet;

use log::trace;
use machine_check_common::{check::Property, StateId};

use crate::space::StateSpace;
use crate::FullMachine;

#[derive(Debug, Clone)]

pub struct Focus {
    depth: usize,
    dirty: BTreeSet<StateId>,
    affected_forward: BTreeSet<StateId>,
    affected_backward: BTreeSet<StateId>,
}

impl Focus {
    pub fn new(property: &Property) -> Self {
        let result = Self {
            depth: property.transition_depth(),
            dirty: BTreeSet::new(),
            affected_forward: BTreeSet::new(),
            affected_backward: BTreeSet::new(),
        };
        trace!("Focus depth: {}", result.depth);
        result
    }

    pub fn clear(&mut self) {
        trace!("Cleared focus");
        self.dirty.clear();
        self.affected_forward.clear();
        self.affected_backward.clear();
    }

    pub fn regenerate<M: FullMachine>(&mut self, space: &StateSpace<M>, added: &BTreeSet<StateId>) {
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
            self.affected_backward.remove(state);
            self.dirty.remove(state);
            self.affected_forward.remove(state);
        }
    }

    pub fn dirty(&self) -> &BTreeSet<StateId> {
        &self.dirty
    }

    pub fn dirty_iter(&self) -> impl Iterator<Item = StateId> + use<'_> {
        self.dirty().iter().copied()
    }

    pub fn affected_forward(&self) -> &BTreeSet<StateId> {
        &self.affected_forward
    }

    pub fn affected_backward(&self) -> &BTreeSet<StateId> {
        &self.affected_backward
    }

    pub fn make_whole_dirty<M: FullMachine>(&mut self, space: &StateSpace<M>) {
        for state_id in space.states() {
            self.insert_dirty(space, state_id);
        }
    }

    pub fn insert_dirty<M: FullMachine>(&mut self, space: &StateSpace<M>, state_id: StateId) {
        let newly_inserted = self.dirty.insert(state_id);
        if !newly_inserted {
            // no change
            return;
        }
        // make sure the affected states correspond

        self.ensure_forward(space, state_id);
        self.ensure_backward(space, state_id);
    }

    fn ensure_forward<M: FullMachine>(&mut self, space: &StateSpace<M>, state_id: StateId) {
        self.affected_forward.insert(state_id);
        let mut current_affected = BTreeSet::from([state_id]);
        let mut next_affected = BTreeSet::new();

        for _ in 0..self.depth {
            for state_id in current_affected.iter().copied() {
                for direct_successor_id in space.direct_successor_iter(state_id.into()) {
                    self.affected_forward.insert(direct_successor_id);
                    next_affected.insert(direct_successor_id);
                }
            }
            current_affected.clear();
            std::mem::swap(&mut current_affected, &mut next_affected);
        }
    }

    fn ensure_backward<M: FullMachine>(&mut self, space: &StateSpace<M>, state_id: StateId) {
        self.affected_backward.insert(state_id);
        let mut current_affected = BTreeSet::from([state_id]);
        let mut next_affected = BTreeSet::new();

        for _ in 0..self.depth {
            for state_id in current_affected.iter().copied() {
                for direct_predecessor_id in space.direct_predecessor_iter(state_id.into()) {
                    if let Ok(direct_predecessor_id) = StateId::try_from(direct_predecessor_id) {
                        self.affected_backward.insert(direct_predecessor_id);
                        next_affected.insert(direct_predecessor_id);
                    }
                }
            }
            current_affected.clear();
            std::mem::swap(&mut current_affected, &mut next_affected);
        }
    }
}
