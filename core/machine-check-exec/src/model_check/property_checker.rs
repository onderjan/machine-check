mod labelling_computer;

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Debug,
};

use machine_check_common::{check::Property, ExecError, StateId, ThreeValued};
use mck::concr::FullMachine;

use crate::{
    model_check::property_checker::labelling_computer::LabellingComputer, space::StateSpace,
};

#[derive(Debug)]
pub struct PropertyChecker {
    cache: BTreeMap<usize, BTreeMap<StateId, CheckValue>>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct CheckValue {
    pub valuation: ThreeValued,
    pub next_states: Vec<StateId>,
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
    pub fn new() -> Self {
        Self {
            cache: BTreeMap::new(),
        }
    }

    pub fn purge_states(&mut self, _purge_states: &BTreeSet<StateId>) {
        self.cache.clear();
        // TODO: incremental
        /*self.very_dirty.extend(purge_states.iter());

        for check_info in self.subproperty_map.values_mut() {
            check_info.dirty.extend(purge_states.iter());

            for state_id in purge_states {
                check_info.labelling.remove(state_id);
            }
        }*/
    }

    pub fn compute_interpretation<M: FullMachine>(
        &mut self,
        space: &StateSpace<M>,
        property: &Property,
    ) -> Result<ThreeValued, ExecError> {
        let mut labelling_computer = LabellingComputer::new(self, property, space)?;
        let result = labelling_computer.compute()?;
        let mut cache = BTreeMap::new();
        for subproperty_index in 0..property.num_subproperties() {
            let values = labelling_computer
                .subproperty_values(subproperty_index)
                .clone();
            cache.insert(subproperty_index, values);
        }

        self.cache = cache;

        Ok(result)
    }

    pub fn get_state_label(&self, subproperty_index: usize, state_index: StateId) -> &CheckValue {
        // TODO: this is wasteful when looking at multiple states
        self.get_labelling(subproperty_index)
            .get(&state_index)
            .expect("Should contain state labelling")
    }

    pub fn get_state_root_label(&self, state_index: StateId) -> &CheckValue {
        self.get_state_label(0, state_index)
    }

    pub fn get_labelling(&self, subproperty_index: usize) -> &BTreeMap<StateId, CheckValue> {
        self.cache
            .get(&subproperty_index)
            .expect("Labelling should be present")
    }
}
