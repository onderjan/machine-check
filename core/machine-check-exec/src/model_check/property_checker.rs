mod double_check;
mod focus;
mod history;
mod labelling_cacher;
mod labelling_updater;

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
        labelling_updater::LabellingUpdater,
    },
    space::StateSpace,
};

pub use labelling_cacher::BiChoice;
pub use labelling_cacher::LabellingCacher;

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
        let labelling_computer = LabellingUpdater::new(self, space)?;
        let result = labelling_computer.compute()?;

        trace!(
            "Histories after computing interpretation: {:#?}",
            self.histories
        );

        // double-check to be sure

        Ok(result)
    }

    pub fn last_getter<'a, M: FullMachine>(
        &'a self,
        space: &'a StateSpace<M>,
    ) -> LabellingCacher<'a, M> {
        LabellingCacher::new(self, space, u64::MAX)
    }

    fn invalidate(&mut self) {
        for history in self.histories.values_mut() {
            history.clear();
        }
        self.computations.clear();
    }

    fn squash(&mut self) -> Result<(), ExecError> {
        //self.squash_inner(&mut 0, &mut 0, 0)

        let mut update_times = BTreeSet::new();

        for history in self.histories.values() {
            update_times.extend(history.time_keys())
        }

        let mut time_subtracts = BTreeMap::new();

        let after_last_time = *update_times
            .last()
            .expect("There should be at least one update time")
            + 1;

        for (squash_time, update_time) in update_times.into_iter().enumerate() {
            let subtract = update_time - squash_time as u64;

            time_subtracts.insert(update_time, subtract);
        }

        for history in self.histories.values_mut() {
            history.squash(&time_subtracts, after_last_time);
        }

        let mut computations = Vec::new();
        std::mem::swap(&mut computations, &mut self.computations);

        for mut computation in computations {
            if computation.start_time >= after_last_time {
                // this computation does not do anything
                break;
            }
            computation.start_time =
                squash_time(&time_subtracts, after_last_time, computation.start_time);
            computation.end_time =
                squash_time(&time_subtracts, after_last_time, computation.end_time);
            self.computations.push(computation);
        }

        Ok(())
    }
}

fn squash_time(
    time_subtracts: &BTreeMap<u64, u64>,
    after_last_time: u64,
    mut original_time: u64,
) -> u64 {
    if original_time > after_last_time {
        original_time = after_last_time;
    }

    let subtract = *time_subtracts
        .range(..=original_time)
        .next_back()
        .expect("Original time should have a time subtract")
        .1;

    original_time - subtract
}
