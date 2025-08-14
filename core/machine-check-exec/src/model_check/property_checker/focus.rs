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
}

impl Focus {
    pub fn new(property: &Property) -> Self {
        let result = Self {
            depth: property.transition_depth(),
            dirty: BTreeSet::new(),
            affected_forward: BTreeSet::new(),
        };
        trace!("Focus depth: {}", result.depth);
        result
    }

    pub fn clear(&mut self) {
        trace!("Cleared focus");
        self.dirty.clear();
        self.affected_forward.clear();
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

        self.affected_forward.insert(state_id);

        let mut current_affected_forward = BTreeSet::from([state_id]);
        let mut next_affected_forward = BTreeSet::new();

        for _ in 0..self.depth {
            for state_id in current_affected_forward.iter().copied() {
                for direct_successor_id in space.direct_successor_iter(state_id.into()) {
                    self.affected_forward.insert(direct_successor_id);
                    next_affected_forward.insert(direct_successor_id);
                }
            }
            current_affected_forward.clear();
            std::mem::swap(&mut current_affected_forward, &mut next_affected_forward);
        }

        /*trace!(
            "Inserted dirty state {}, newly affected: {:?}",
            state_id, self.affected
        );*/
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
}
