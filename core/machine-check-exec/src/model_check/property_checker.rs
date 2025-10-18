mod double_check;
mod focus;
mod history;
mod labelling_cacher;
mod labelling_updater;
mod value;

use std::{collections::BTreeMap, fmt::Debug};

use log::trace;
use machine_check_common::{
    iir::{IProperty, ISubproperty},
    ExecError, ParamValuation,
};
use mck::concr::FullMachine;

pub use labelling_cacher::LabellingCacher;
pub(super) use value::{CheckChoice, CheckValue};

use crate::{
    model_check::property_checker::{
        focus::Focus, history::FixedPointHistory, labelling_updater::LabellingUpdater,
    },
    space::StateSpace,
};

#[derive(Debug, Clone)]
pub struct PropertyChecker {
    property: IProperty,

    // TODO: re-add closed-form subproperties
    //closed_form_subproperties: BTreeSet<usize>,
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
    pub fn new(property: IProperty) -> Self {
        //let mut closed_form_subproperties = BTreeSet::new();
        let mut histories = BTreeMap::new();

        for subproperty_index in 0..property.num_subproperties() {
            /*if property.is_subproperty_closed_form(subproperty_index) {
                closed_form_subproperties.insert(subproperty_index);
            }*/

            let subproperty = property.subproperty_entry(subproperty_index);
            if matches!(subproperty, ISubproperty::FixedPoint(_)) {
                histories.insert(subproperty_index, FixedPointHistory::default());
            }
        }

        let focus = Focus::new(&property);

        Self {
            property,
            //closed_form_subproperties,
            focus,
            histories,
            computations: Vec::new(),
        }
    }

    /*pub fn purge_states<M: FullMachine>(
        &mut self,
        space: &StateSpace<M>,
        purge_states: &BTreeSet<StateId>,
    ) {
        self.focus.regenerate(space, purge_states);
    }

    pub fn remove_states(&mut self, removed_states: &BTreeSet<StateId>) {
        self.focus.remove_states(removed_states);
        for history in self.histories.values_mut() {
            history.remove_states(removed_states)
        }
    }*/

    pub fn compute_interpretation<M: FullMachine>(
        &mut self,
        space: &StateSpace<M>,
    ) -> Result<ParamValuation, ExecError> {
        // TODO: do not clear histories and make whole dirty
        for history in self.histories.values_mut() {
            history.clear();
        }
        self.focus.make_whole_dirty(space);
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

        Ok(result)
    }

    pub fn last_getter<'a, M: FullMachine>(
        &'a self,
        space: &'a StateSpace<M>,
    ) -> LabellingCacher<'a, M> {
        LabellingCacher::new(self, space)
    }

    fn invalidate(&mut self) {
        for history in self.histories.values_mut() {
            history.clear();
        }
        self.computations.clear();
    }

    pub fn get_history(&self, fixed_point_index: usize) -> &FixedPointHistory {
        self.histories
            .get(&fixed_point_index)
            .expect("History should exist")
    }
}
