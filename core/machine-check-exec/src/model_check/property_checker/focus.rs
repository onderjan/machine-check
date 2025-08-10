use std::collections::BTreeSet;

use log::trace;
use machine_check_common::{check::Property, StateId};

use crate::space::StateSpace;
use crate::FullMachine;

#[derive(Debug, Clone)]

pub struct Focus {
    depth: usize,
    dirty: BTreeSet<StateId>,
    affected: BTreeSet<StateId>,
}

impl Focus {
    pub fn new(property: &Property) -> Self {
        let result = Self {
            depth: property.transition_depth(),
            dirty: BTreeSet::new(),
            affected: BTreeSet::new(),
        };
        trace!("Focus depth: {}", result.depth);
        result
    }

    pub fn clear(&mut self) {
        self.dirty.clear();
        self.affected.clear();
    }

    pub fn dirty(&self) -> &BTreeSet<StateId> {
        &self.dirty
    }

    pub fn dirty_iter(&self) -> impl Iterator<Item = StateId> + use<'_> {
        self.dirty().iter().copied()
    }

    pub fn affected(&self) -> &BTreeSet<StateId> {
        &self.affected
    }

    pub fn extend_dirty<M: FullMachine>(
        &mut self,
        space: &StateSpace<M>,
        iter: impl Iterator<Item = StateId>,
    ) {
        for state_id in iter {
            self.insert_dirty(space, state_id);
        }
    }

    pub fn insert_dirty<M: FullMachine>(&mut self, space: &StateSpace<M>, state_id: StateId) {
        if self.dirty.contains(&state_id) {
            // already dirty
            return;
        }

        self.dirty.insert(state_id);
        self.affected.insert(state_id);

        let mut current_affected = BTreeSet::from([state_id]);
        let mut next_affected = BTreeSet::new();

        for _ in 0..self.depth {
            for state_id in current_affected.iter().copied() {
                for direct_successor_id in space.direct_successor_iter(state_id.into()) {
                    self.affected.insert(direct_successor_id);
                    next_affected.insert(direct_successor_id);
                }
            }
            current_affected.clear();
            current_affected.append(&mut next_affected);
        }
    }

    pub fn regenerate<M: FullMachine>(&mut self, space: &StateSpace<M>, added: &BTreeSet<StateId>) {
        let mut dirty = BTreeSet::new();
        std::mem::swap(&mut dirty, &mut self.dirty);
        dirty.extend(added);
        self.affected.clear();

        self.extend_dirty(space, dirty.iter().copied());
    }
}
