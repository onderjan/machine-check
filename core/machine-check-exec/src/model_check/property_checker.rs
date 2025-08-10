mod focus;
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
        focus::Focus,
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
        self.focus.regenerate(space, purge_states);
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
