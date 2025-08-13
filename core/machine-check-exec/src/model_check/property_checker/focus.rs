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

    pub fn extend_dirty<M: FullMachine>(
        &mut self,
        space: &StateSpace<M>,
        iter: impl Iterator<Item = StateId>,
    ) {
        trace!("Extending dirty");
        for state_id in iter {
            self.insert_dirty(space, state_id);
        }

        trace!("Extended dirty");
    }

    pub fn insert_dirty<M: FullMachine>(&mut self, space: &StateSpace<M>, state_id: StateId) {
        self.dirty.insert(state_id);
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
            current_affected_forward.append(&mut next_affected_forward);
        }

        let mut current_affected_backward = BTreeSet::from([state_id]);
        let mut next_affected_backward = BTreeSet::new();

        for _ in 0..self.depth {
            for state_id in current_affected_backward.iter().copied() {
                for direct_predecessor_id in space.direct_predecessor_iter(state_id.into()) {
                    let Ok(direct_predecessor_id) = StateId::try_from(direct_predecessor_id) else {
                        continue;
                    };

                    self.affected_backward.insert(direct_predecessor_id);
                    next_affected_backward.insert(direct_predecessor_id);
                }
            }
            current_affected_backward.clear();
            current_affected_backward.append(&mut next_affected_backward);
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

        self.extend_dirty(space, dirty.iter().copied());
    }
}
