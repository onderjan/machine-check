use std::collections::{BTreeSet, HashMap};

use log::{log_enabled, trace};
use machine_check_common::{
    property::{BiOperator, OperatorG, OperatorU, Property, TemporalOperator, UniOperator},
    ExecError, StateId,
};
use mck::concr::FullMachine;

use crate::space::StateSpace;

/// Two-valued model-checker for the state space.
///
/// Acts as one part of a three-valued model-checker based on the value of `optimistic`.
pub struct ClassicChecker<'a, M: FullMachine> {
    space: &'a StateSpace<M>,
    optimistic: bool,
    labelling_map: HashMap<Property, BTreeSet<StateId>>,
}

impl<'a, M: FullMachine> ClassicChecker<'a, M> {
    /// Classic two-valued model checker with chosen interpretation of unknown labellings.
    pub fn new(space: &'a StateSpace<M>, optimistic: bool) -> Self {
        ClassicChecker {
            space,
            optimistic,
            labelling_map: HashMap::new(),
        }
    }

    pub fn compute_interpretation(&mut self, prop: &Property) -> Result<bool, ExecError> {
        self.compute_labelling(prop)?;
        let labelling = self.get_labelling(prop);
        // conventionally, the property must hold in all initial states
        for initial_index in self.space.initial_iter() {
            if !labelling.contains(&initial_index) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub fn get_labelling(&self, prop: &Property) -> &BTreeSet<StateId> {
        self.labelling_map
            .get(prop)
            .expect("labelling should be present")
    }

    pub fn compute_and_get_labelling(
        &mut self,
        prop: &Property,
    ) -> Result<&BTreeSet<StateId>, ExecError> {
        self.compute_labelling(prop)?;
        Ok(self.get_labelling(prop))
    }

    fn compute_labelling(&mut self, prop: &Property) -> Result<(), ExecError> {
        if self.labelling_map.contains_key(prop) {
            // already contained
            return Ok(());
        }

        let computed_labelling = match prop {
            Property::Const(c) => {
                if *c {
                    // holds in all state indices
                    BTreeSet::from_iter(
                        self.space
                            .nodes()
                            .filter_map(|node_id| StateId::try_from(node_id).ok()),
                    )
                } else {
                    // holds nowhere
                    BTreeSet::new()
                }
            }
            Property::Atomic(atomic_property) => {
                // get from space
                let labelled: Result<BTreeSet<_>, ()> = self
                    .space
                    .labelled_iter(atomic_property, self.optimistic)
                    .collect();
                match labelled {
                    Ok(labelled) => labelled,
                    Err(_) => {
                        return Err(ExecError::FieldNotFound(String::from(
                            atomic_property.left().name(),
                        )))
                    }
                }
            }
            Property::Negation(inner) => {
                // complement
                let full_labelling = BTreeSet::from_iter(
                    self.space
                        .nodes()
                        .filter_map(|node_id| StateId::try_from(node_id).ok()),
                );
                self.compute_labelling(&inner.0)?;
                let inner_labelling = self.get_labelling(&inner.0);
                full_labelling
                    .difference(inner_labelling)
                    .cloned()
                    .collect()
            }
            Property::Or(BiOperator { a, b }) => {
                self.compute_labelling(a)?;
                self.compute_labelling(b)?;
                let a_labelling = self.get_labelling(a);
                let b_labelling = self.get_labelling(b);
                a_labelling.union(b_labelling).cloned().collect()
            }
            Property::And(BiOperator { a, b }) => {
                self.compute_labelling(a)?;
                self.compute_labelling(b)?;
                let a_labelling = self.get_labelling(a);
                let b_labelling = self.get_labelling(b);
                a_labelling.intersection(b_labelling).cloned().collect()
            }
            Property::E(prop_temp) => match prop_temp {
                TemporalOperator::X(inner) => self.compute_ex_labelling(inner)?,
                TemporalOperator::G(inner) => self.compute_eg_labelling(inner)?,
                TemporalOperator::U(inner) => self.compute_eu_labelling(inner)?,
                _ => {
                    panic!(
                        "expected {:?} to have only X, G, U temporal operators",
                        prop
                    );
                }
            },
            _ => panic!("expected {:?} to be minimized", prop),
        };

        if log_enabled!(log::Level::Trace) {
            trace!(
                "{}: computed property {:?} labelling {:?}",
                self.optimistic,
                prop,
                computed_labelling
            );
        }

        // insert the labelling to labelling map for future reference
        self.labelling_map.insert(prop.clone(), computed_labelling);

        Ok(())
    }

    fn compute_ex_labelling(
        &mut self,
        inner: &UniOperator,
    ) -> Result<BTreeSet<StateId>, ExecError> {
        self.compute_labelling(&inner.0)?;
        let inner_labelling = self.get_labelling(&inner.0);
        let mut result = BTreeSet::new();
        // for each state labelled by p, mark the preceding states EX[p]
        for state_id in inner_labelling {
            for direct_predecessor_id in self.space.direct_predecessor_iter((*state_id).into()) {
                // ignore start node
                if let Ok(direct_predecessor_id) = StateId::try_from(direct_predecessor_id) {
                    result.insert(direct_predecessor_id);
                }
            }
        }
        Ok(result)
    }

    fn compute_eg_labelling(&mut self, inner: &OperatorG) -> Result<BTreeSet<StateId>, ExecError> {
        // Boolean SCC-based labelling procedure CheckEG from Model Checking 1999 by Clarke et al.

        // compute inner labelling
        self.compute_labelling(&inner.0)?;
        let inner_labelling = self.get_labelling(&inner.0);

        // compute states of nontrivial strongly connected components of labelled and insert them into working set
        let mut working_set = self.space.labelled_nontrivial_scc_indices(inner_labelling);

        // make all states in working set labelled EG(f)
        let mut eg_labelling = working_set.clone();

        // choose and process states from working set until empty
        while let Some(state_id) = working_set.pop_first() {
            // for every directed predecessor of the chosen state which is labelled (f) but not EG(f) yet,
            // label it EG f and add to the working set
            for previous_id in self.space.direct_predecessor_iter(state_id.into()) {
                // ignore start node
                if let Ok(previous_id) = StateId::try_from(previous_id) {
                    if inner_labelling.contains(&previous_id) {
                        let inserted = eg_labelling.insert(previous_id);
                        if inserted {
                            working_set.insert(previous_id);
                        }
                    }
                }
            }
        }

        // return states labelled EG(f)
        Ok(eg_labelling)
    }

    fn compute_eu_labelling(&mut self, prop: &OperatorU) -> Result<BTreeSet<StateId>, ExecError> {
        // worklist-based labelling procedure CheckEU from Model Checking 1999 by Clarke et al.

        self.compute_labelling(&prop.hold)?;
        self.compute_labelling(&prop.until)?;

        let prop = prop.clone();

        let hold_labelling = self.get_labelling(&prop.hold);
        let until_labelling = self.get_labelling(&prop.until);

        // the working set holds all states labeled "until" at the start
        let mut working = until_labelling.clone();
        // make all states in working set labelled EU(f,g)
        let mut eu_labelling = working.clone();

        // choose and process states from working set until empty
        while let Some(state_index) = working.pop_first() {
            // for every parent of the chosen state which is labeled (f) but not EU(f,g) yet,
            // label it EU(f,g) and add to the working set
            for previous_id in self.space.direct_predecessor_iter(state_index.into()) {
                // ignore start node
                if let Ok(previous_id) = StateId::try_from(previous_id) {
                    if hold_labelling.contains(&previous_id) {
                        let inserted = eu_labelling.insert(previous_id);
                        if inserted {
                            working.insert(previous_id);
                        }
                    }
                }
            }
        }

        Ok(eu_labelling)
    }
}
