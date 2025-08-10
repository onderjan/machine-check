mod history;
mod labelling_computer;
mod labelling_getter;

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Debug,
};

use log::trace;
use machine_check_common::{
    check::Property, property::PropertyType, ExecError, StateId, ThreeValued,
};
use mck::concr::FullMachine;

use crate::{
    model_check::property_checker::{
        history::{CheckValue, FixedPointHistory, TimedCheckValue},
        labelling_computer::LabellingComputer,
    },
    space::StateSpace,
};

pub use labelling_getter::BiChoice;
pub use labelling_getter::LabellingGetter;

#[derive(Debug, Clone)]
pub struct PropertyChecker {
    property: Property,
    closed_form_subproperties: BTreeSet<usize>,

    histories: BTreeMap<usize, FixedPointHistory>,
    computations: Vec<FixedPointComputation>,

    focus: Focus,
}

#[derive(Debug, Clone)]

struct Focus {
    depth: usize,
    dirty: BTreeSet<StateId>,
    affected: BTreeSet<StateId>,
}

impl Focus {
    fn new(property: &Property) -> Self {
        let result = Self {
            depth: property.transition_depth(),
            dirty: BTreeSet::new(),
            affected: BTreeSet::new(),
        };
        trace!("Focus depth: {}", result.depth);
        result
    }

    fn clear(&mut self) {
        self.dirty.clear();
        self.affected.clear();
    }

    fn dirty(&self) -> &BTreeSet<StateId> {
        &self.dirty
    }

    fn dirty_iter(&self) -> impl Iterator<Item = StateId> + use<'_> {
        self.dirty().iter().copied()
    }

    fn affected(&self) -> &BTreeSet<StateId> {
        &self.affected
    }

    fn extend_dirty<M: FullMachine>(
        &mut self,
        space: &StateSpace<M>,
        iter: impl Iterator<Item = StateId>,
    ) {
        for state_id in iter {
            self.insert_dirty(space, state_id);
        }
    }

    fn insert_dirty<M: FullMachine>(&mut self, space: &StateSpace<M>, state_id: StateId) {
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
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct FixedPointComputation {
    pub fixed_point_index: usize,
    pub start_time: u64,
    pub end_time: u64,
}

impl PropertyChecker {
    pub fn new(property: Property) -> Self {
        let mut closed_form_subproperties = BTreeSet::new();
        let mut histories = BTreeMap::new();

        for subproperty_index in 0..property.num_subproperties() {
            if property.is_subproperty_closed_form(subproperty_index) {
                closed_form_subproperties.insert(subproperty_index);
            }

            let subproperty = property.subproperty_entry(subproperty_index);
            if matches!(subproperty.ty, PropertyType::FixedPoint(_)) {
                histories.insert(subproperty_index, FixedPointHistory::default());
            }
        }

        let focus = Focus::new(&property);

        Self {
            property,
            closed_form_subproperties,
            focus,
            histories,
            computations: Vec::new(),
        }
    }

    pub fn purge_states<M: FullMachine>(
        &mut self,
        space: &StateSpace<M>,
        purge_states: &BTreeSet<StateId>,
    ) {
        self.focus.extend_dirty(space, purge_states.iter().copied());
    }

    pub fn compute_interpretation<M: FullMachine>(
        &mut self,
        space: &StateSpace<M>,
    ) -> Result<ThreeValued, ExecError> {
        trace!(
            "Histories before computing interpretation: {:#?}",
            self.histories
        );
        let labelling_computer = LabellingComputer::new(self, space)?;
        let result = labelling_computer.compute()?;

        trace!(
            "Histories after computing interpretation: {:#?}",
            self.histories
        );

        Ok(result)
    }

    pub fn last_getter<'a, M: FullMachine>(
        &'a self,
        space: &'a StateSpace<M>,
    ) -> LabellingGetter<'a, M> {
        LabellingGetter::new(self, space, u64::MAX)
    }

    fn invalidate(&mut self) {
        for history in self.histories.values_mut() {
            history.clear();
        }
        self.computations.clear();
    }
}
