mod labelling_computer;

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
    model_check::property_checker::labelling_computer::LabellingComputer, space::StateSpace,
};

#[derive(Debug)]
pub struct PropertyChecker {
    property: Property,

    recompute_states: BTreeSet<StateId>,
    fixed_point_histories: BTreeMap<usize, FixedPointHistory>,
    latest: BTreeMap<usize, BTreeMap<StateId, CheckValue>>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct CheckValue {
    pub valuation: ThreeValued,
    pub next_states: Vec<StateId>,
}

#[derive(Clone, Debug, Default)]
struct FixedPointHistory {
    times: BTreeMap<u64, BTreeMap<StateId, CheckValue>>,
    states: BTreeMap<StateId, BTreeMap<u64, CheckValue>>,
}

impl FixedPointHistory {
    fn insert_update(&mut self, time_instant: u64, state_id: StateId, value: CheckValue) {
        // TODO
        self.times
            .entry(time_instant)
            .or_default()
            .insert(state_id, value.clone());

        self.states
            .entry(state_id)
            .or_default()
            .insert(time_instant, value);
    }

    /*fn latest(&self, state_id: StateId) -> &CheckValue {
        self.states
            .get(&state_id)
            .expect("State should have history")
            .last_key_value()
            .expect("State should have latest")
            .1
    }

    fn before(&self, time_instant: u64, state_id: StateId) -> Option<&CheckValue> {
        self.states
            .get(&state_id)
            .expect("State should have history")
            .range(0..time_instant)
            .last()
            .map(|(_last_instant, check_value)| check_value)
    }*/
}

impl CheckValue {
    pub fn eigen(value: ThreeValued) -> Self {
        Self {
            valuation: value,
            next_states: vec![],
        }
    }
}

impl Debug for CheckValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {:?}", self.valuation, self.next_states)
    }
}

impl PropertyChecker {
    pub fn new(property: Property) -> Self {
        Self {
            property,
            recompute_states: BTreeSet::new(),
            fixed_point_histories: BTreeMap::new(),
            latest: BTreeMap::new(),
        }
    }

    pub fn purge_states(&mut self, purge_states: &BTreeSet<StateId>) {
        self.recompute_states.extend(purge_states);
    }

    pub fn compute_interpretation<M: FullMachine>(
        &mut self,
        space: &StateSpace<M>,
    ) -> Result<ThreeValued, ExecError> {
        self.recompute_states.extend(space.states());

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

        trace!(
            "Latest labellings after computing interpretation: {:#?}",
            self.latest
        );

        Ok(result)
    }

    fn reinit_labellings(&mut self) -> Result<(), ExecError> {
        // do latest
        self.latest.clear();

        for subproperty_index in 0..self.property.num_subproperties() {
            self.latest.insert(subproperty_index, BTreeMap::new());
        }

        // do fixed-point histories
        self.fixed_point_histories.clear();

        for subproperty_index in 0..self.property.num_subproperties() {
            if matches!(
                self.property.subproperty_entry(subproperty_index).ty,
                PropertyType::FixedPoint(_)
            ) {
                self.fixed_point_histories
                    .insert(subproperty_index, FixedPointHistory::default());
            }
        }

        Ok(())
    }

    pub fn get_state_label(&self, subproperty_index: usize, state_index: StateId) -> &CheckValue {
        self.get_labelling(subproperty_index)
            .get(&state_index)
            .expect("Should contain state labelling")
    }

    pub fn get_state_root_label(&self, state_index: StateId) -> &CheckValue {
        self.get_state_label(0, state_index)
    }

    pub fn get_labelling(&self, subproperty_index: usize) -> &BTreeMap<StateId, CheckValue> {
        self.latest
            .get(&subproperty_index)
            .expect("Labelling should be present")
    }

    pub fn get_labelling_mut(
        &mut self,
        subproperty_index: usize,
    ) -> &mut BTreeMap<StateId, CheckValue> {
        self.latest
            .get_mut(&subproperty_index)
            .expect("Labelling should be present")
    }
}
