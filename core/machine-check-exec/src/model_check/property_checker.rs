mod labelling_computer;

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Debug,
};

use log::trace;
use machine_check_common::{check::Property, ExecError, StateId, ThreeValued};
use mck::concr::FullMachine;

use crate::{
    model_check::property_checker::labelling_computer::LabellingComputer, space::StateSpace,
};

#[derive(Debug)]
pub struct PropertyChecker {
    final_labellings: BTreeMap<usize, BTreeMap<StateId, CheckValue>>,
    old_cache: Vec<CacheEntry>,
    old_cache_index: usize,
    cache: Vec<CacheEntry>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct CheckValue {
    pub valuation: ThreeValued,
    pub next_states: Vec<StateId>,
}

#[derive(Debug)]
struct CacheEntry {
    fixed_point_index: usize,
    time_instant: u64,
    histories: BTreeMap<usize, FixedPointHistory>,
}

// TODO: remove clone
#[derive(Clone, Debug, Default)]
struct FixedPointHistory {
    times: BTreeMap<u64, BTreeMap<StateId, CheckValue>>,
    states: BTreeMap<StateId, BTreeMap<u64, CheckValue>>,
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
            final_labellings: BTreeMap::new(),
            cache: Vec::new(),
            old_cache: Vec::new(),
            old_cache_index: 0,
        }
    }

    pub fn purge_states(&mut self, _purge_states: &BTreeSet<StateId>) {
        self.final_labellings.clear();
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
        trace!("Cache before computing interpretation: {:#?}", self.cache);
        let mut labelling_computer = LabellingComputer::new(self, property, space)?;
        let result = labelling_computer.compute()?;
        let mut final_labellings = BTreeMap::new();
        for subproperty_index in 0..property.num_subproperties() {
            let values = labelling_computer
                .subproperty_values(subproperty_index)
                .clone();
            final_labellings.insert(subproperty_index, values);
        }

        self.final_labellings = final_labellings;

        trace!("Cache after computing interpretation: {:#?}", self.cache);

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
        self.final_labellings
            .get(&subproperty_index)
            .expect("Labelling should be present")
    }
}
