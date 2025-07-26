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

#[derive(Debug)]
pub struct PropertyChecker {
    property: Property,

    dirty_states: BTreeSet<StateId>,
    fixed_point_histories: BTreeMap<usize, FixedPointHistory>,
}

impl PropertyChecker {
    pub fn new(property: Property) -> Self {
        let mut fixed_point_histories = BTreeMap::new();

        for subproperty_index in 0..property.num_subproperties() {
            if matches!(
                property.subproperty_entry(subproperty_index).ty,
                PropertyType::FixedPoint(_)
            ) {
                fixed_point_histories.insert(subproperty_index, FixedPointHistory::default());
            }
        }

        Self {
            property,
            dirty_states: BTreeSet::new(),
            fixed_point_histories,
        }
    }

    pub fn purge_states(&mut self, purge_states: &BTreeSet<StateId>) {
        self.dirty_states.extend(purge_states);
    }

    pub fn compute_interpretation<M: FullMachine>(
        &mut self,
        space: &StateSpace<M>,
    ) -> Result<ThreeValued, ExecError> {
        trace!(
            "Histories before computing interpretation: {:#?}",
            self.fixed_point_histories
        );
        let mut labelling_computer = LabellingComputer::new(self, space)?;
        let result = labelling_computer.compute()?;

        trace!(
            "Histories after computing interpretation: {:#?}",
            self.fixed_point_histories
        );

        Ok(result)
    }

    pub fn last_getter<'a, M: FullMachine>(
        &'a self,
        space: &'a StateSpace<M>,
    ) -> LabellingGetter<'a, M> {
        LabellingGetter::new(self, space, u64::MAX)
    }
}
