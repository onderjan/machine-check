use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};

use log::{log_enabled, trace};
use machine_check_common::{
    property::{
        BiOperator, FixedPointOperator, FixedPointVariable, OperatorG, OperatorU, Property,
        TemporalOperator, UniOperator,
    },
    ExecError, StateId, ThreeValued,
};
use mck::concr::FullMachine;

use crate::space::StateSpace;

/// Two-valued model-checker for the state space.
///
/// Acts as one part of a three-valued model-checker based on the value of `optimistic`.
/// TODO: update the comments
pub struct ClassicChecker<'a, M: FullMachine> {
    space: &'a StateSpace<M>,
    labelling_map: HashMap<Property, BTreeMap<StateId, ThreeValued>>,
    reasons_map: HashMap<Property, BTreeMap<StateId, StateId>>,
}

impl<'a, M: FullMachine> ClassicChecker<'a, M> {
    /// Classic two-valued model checker with chosen interpretation of unknown labellings.
    pub fn new(space: &'a StateSpace<M>) -> Self {
        ClassicChecker {
            space,
            labelling_map: HashMap::new(),
            reasons_map: HashMap::new(),
        }
    }

    pub fn compute_interpretation(&mut self, prop: &Property) -> Result<ThreeValued, ExecError> {
        self.compute_labelling(prop)?;
        let labelling = self.get_labelling(prop);
        // conventionally, the property must hold in all initial states
        let mut result = ThreeValued::True;
        for initial_state_id in self.space.initial_iter() {
            let state_labelling = labelling
                .get(&initial_state_id)
                .expect("Labelling should contain initial state");
            result = result & *state_labelling;
        }
        Ok(result)
    }

    pub fn get_labelling(&self, prop: &Property) -> &BTreeMap<StateId, ThreeValued> {
        let labelling = self.labelling_map.get(prop);
        if let Some(labelling) = labelling {
            labelling
        } else {
            panic!("Labelling should be present for {}", prop);
        }
    }

    pub fn get_reasons(&self, prop: &Property) -> &BTreeMap<StateId, StateId> {
        let result = self.reasons_map.get(prop);
        if let Some(result) = result {
            result
        } else {
            panic!("Reasons should be present for {}", prop);
        }
    }

    pub fn get_reasons_mut(&mut self, prop: &Property) -> &mut BTreeMap<StateId, StateId> {
        self.reasons_map.entry(prop.clone()).or_default()
    }

    pub fn compute_and_get_labelling(
        &mut self,
        prop: &Property,
    ) -> Result<&BTreeMap<StateId, ThreeValued>, ExecError> {
        self.compute_labelling(prop)?;
        Ok(self.get_labelling(prop))
    }

    fn compute_labelling(&mut self, prop: &Property) -> Result<(), ExecError> {
        if self.labelling_map.contains_key(prop) {
            // already contained
            return Ok(());
        }

        let labelling = match prop {
            Property::Const(c) => {
                // constant labelling
                self.constant_labelling(ThreeValued::from_bool(*c))
            }
            Property::Atomic(atomic_property) => {
                // get from space
                let labelled: Result<Vec<(StateId, ThreeValued)>, _> =
                    self.space.labelled_iter(atomic_property).collect();
                BTreeMap::from_iter(labelled?)
            }
            Property::Negation(inner) => {
                // complement
                self.compute_labelling(&inner.0)?;
                let mut result = self.get_labelling(&inner.0).clone();
                for value in result.values_mut() {
                    *value = !*value;
                }
                result
            }
            Property::Or(BiOperator { a, b }) => {
                self.compute_labelling(a)?;
                self.compute_labelling(b)?;
                let a_labelling = self.get_labelling(a);
                let b_labelling = self.get_labelling(b);
                assert_eq!(a_labelling.len(), b_labelling.len());
                let mut result = BTreeMap::new();
                for (state_id, a_value) in a_labelling.iter() {
                    let b_value = b_labelling
                        .get(state_id)
                        .expect("Labelling elements should be the same when performing OR");
                    let result_value = *a_value | *b_value;
                    result.insert(*state_id, result_value);
                }
                result
            }
            Property::And(BiOperator { a, b }) => {
                self.compute_labelling(a)?;
                self.compute_labelling(b)?;
                let a_labelling = self.get_labelling(a);
                let b_labelling = self.get_labelling(b);
                assert_eq!(a_labelling.len(), b_labelling.len());
                let mut result = BTreeMap::new();
                for (state_id, a_value) in a_labelling.iter() {
                    let b_value = b_labelling
                        .get(state_id)
                        .expect("Labelling elements should be the same when performing AND");
                    let result_value = *a_value & *b_value;
                    result.insert(*state_id, result_value);
                }
                result
            }
            Property::E(prop_temp) => match prop_temp {
                TemporalOperator::X(inner) => self.compute_next_labelling(inner, false)?,
                TemporalOperator::G(inner) => self.compute_eg_labelling(inner)?,
                TemporalOperator::U(inner) => self.compute_eu_labelling(inner)?,
                _ => {
                    panic!(
                        "expected {:?} to have only X, G, U temporal operators",
                        prop
                    );
                }
            },
            Property::LeastFixedPoint(operator) => {
                self.compute_fixed_point(operator, ThreeValued::from_bool(false))?
            }
            Property::GreatestFixedPoint(operator) => {
                self.compute_fixed_point(operator, ThreeValued::from_bool(true))?
            }
            Property::FixedPointVariable(_) => {
                // the variable has been initialised / computed within the fixed-point operators
                assert!(self.labelling_map.contains_key(prop));
                return Ok(());
            }
            _ => panic!("expected {:?} to be minimized", prop),
        };

        if log_enabled!(log::Level::Trace) {
            trace!("computed property {} labelling {:?}", prop, labelling);
        }

        //println!("Computed property {} labelling {:?}", prop, labelling);
        //println!("Space: {:?}", self.space);
        // insert the labelling to labelling map for future reference
        self.labelling_map.insert(prop.clone(), labelling);

        Ok(())
    }

    fn compute_fixed_point(
        &mut self,
        operator: &FixedPointOperator,
        initial_value: ThreeValued,
    ) -> Result<BTreeMap<StateId, ThreeValued>, ExecError> {
        // initialise variable labelling
        self.labelling_map.insert(
            Property::FixedPointVariable(operator.variable.clone()),
            self.constant_labelling(initial_value),
        );

        // compute inner property labelling and update variable labelling until they match
        loop {
            self.compute_labelling(&operator.inner)?;
            let current_labelling = self.get_labelling(&operator.inner).clone();
            let previous_labelling =
                self.get_labelling(&Property::FixedPointVariable(operator.variable.clone()));
            if previous_labelling == &current_labelling {
                break Ok(current_labelling);
            }
            let variable = operator.variable.clone();
            self.clear_affected_labellings(&variable, &operator.inner);
            self.labelling_map.insert(
                Property::FixedPointVariable(variable),
                current_labelling.clone(),
            );
        }
    }

    fn clear_affected_labellings(
        &mut self,
        variable: &FixedPointVariable,
        prop: &Property,
    ) -> bool {
        let affected = match prop {
            Property::Const(_) => false,
            Property::Atomic(_) => false,
            Property::Negation(uni_operator) => {
                self.clear_affected_labellings(variable, &uni_operator.0)
            }
            Property::Or(bi_operator) | Property::And(bi_operator) => {
                self.clear_affected_labellings(variable, &bi_operator.a)
                    || self.clear_affected_labellings(variable, &bi_operator.b)
            }
            Property::E(temporal_operator) | Property::A(temporal_operator) => {
                let mut affected = false;
                for child in temporal_operator.children() {
                    affected |= self.clear_affected_labellings(variable, &child);
                }
                affected
            }
            Property::LeastFixedPoint(fixed_point_operator)
            | Property::GreatestFixedPoint(fixed_point_operator) => {
                self.clear_affected_labellings(variable, &fixed_point_operator.inner)
            }
            Property::FixedPointVariable(fixed_point_variable) => fixed_point_variable == variable,
        };
        if affected {
            self.labelling_map.remove(prop);
        }
        affected
    }

    fn compute_next_labelling(
        &mut self,
        inner: &UniOperator,
        universal: bool,
    ) -> Result<BTreeMap<StateId, ThreeValued>, ExecError> {
        let prop = TemporalOperator::X(inner.clone());
        let prop = if universal {
            Property::A(prop)
        } else {
            Property::E(prop)
        };
        let mut reasons = self.get_reasons_mut(&prop).clone();

        self.compute_labelling(&inner.0)?;
        let inner_labelling = self.get_labelling(&inner.0);
        //println!("Previous reasons: {:?}", reasons);

        let mut labelling = self.constant_labelling(ThreeValued::from_bool(universal));

        // for each state labelled by p, mark the preceding states X[p]
        for (state_id, value) in inner_labelling {
            if matches!(value, ThreeValued::False) {
                continue;
            }
            for direct_predecessor_id in self.space.direct_predecessor_iter((*state_id).into()) {
                // ignore start node
                let Ok(direct_predecessor_id) = StateId::try_from(direct_predecessor_id) else {
                    continue;
                };
                let direct_predecessor_value = labelling
                    .get_mut(&direct_predecessor_id)
                    .expect("Direct predecessor should be in labelling");
                if universal {
                    *direct_predecessor_value = *direct_predecessor_value & *value;
                } else {
                    *direct_predecessor_value = *direct_predecessor_value | *value;
                }

                reasons.entry(direct_predecessor_id).or_insert(*state_id);
            }
        }

        //println!("Next valuations: {:?}", labelling);
        //println!("Next reasons: {:?}", reasons);

        *self.get_reasons_mut(&prop) = reasons;

        Ok(labelling)
    }

    fn compute_eg_labelling(
        &mut self,
        inner: &OperatorG,
    ) -> Result<BTreeMap<StateId, ThreeValued>, ExecError> {
        // Boolean SCC-based labelling procedure CheckEG from Model Checking 1999 by Clarke et al.

        // compute inner labelling
        self.compute_labelling(&inner.0)?;
        let inner_labelling = self.get_labelling(&inner.0);

        let pessimistic_inner = inner_labelling
            .iter()
            .filter_map(|(state_id, value)| {
                if value.is_true() {
                    Some(*state_id)
                } else {
                    None
                }
            })
            .collect();
        let optimistic_inner = inner_labelling
            .iter()
            .filter_map(|(state_id, value)| {
                if !value.is_false() {
                    Some(*state_id)
                } else {
                    None
                }
            })
            .collect();

        let pessimistic_eg = self.compute_eg_labelling_half(pessimistic_inner);
        let optimistic_eg = self.compute_eg_labelling_half(optimistic_inner);

        let eg_labelling = BTreeMap::from_iter(self.space.nodes().filter_map(|node_id| {
            let Ok(state_id) = StateId::try_from(node_id) else {
                return None;
            };
            let in_pessimistic = pessimistic_eg.contains(&state_id);
            let in_optimistic = optimistic_eg.contains(&state_id);
            let value = match (in_pessimistic, in_optimistic) {
                (true, true) => ThreeValued::True,
                (true, false) => panic!("Value should not be in pessimistic but not in optimistic"),
                (false, true) => ThreeValued::Unknown,
                (false, false) => ThreeValued::False,
            };

            Some((state_id, value))
        }));

        // return states labelled EG(f)
        Ok(eg_labelling)
    }

    fn compute_eg_labelling_half(
        &mut self,
        inner_labelling: BTreeSet<StateId>,
    ) -> BTreeSet<StateId> {
        // compute states of nontrivial strongly connected components of labelled and insert them into working set
        let mut working_set = self.space.labelled_nontrivial_scc_indices(&inner_labelling);

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

        eg_labelling
    }

    fn compute_eu_labelling(
        &mut self,
        prop: &OperatorU,
    ) -> Result<BTreeMap<StateId, ThreeValued>, ExecError> {
        // worklist-based labelling procedure CheckEU from Model Checking 1999 by Clarke et al.

        self.compute_labelling(&prop.hold)?;
        self.compute_labelling(&prop.until)?;

        let hold_labelling = self.get_labelling(&prop.hold);
        let until_labelling = self.get_labelling(&prop.until);

        // the working set holds all states where "until" is labelled true or unknown at the start
        let mut working: VecDeque<StateId> = until_labelling
            .iter()
            .filter_map(|(state_id, value)| {
                if value.is_false() {
                    None
                } else {
                    Some(*state_id)
                }
            })
            .collect();
        // copy the until labelling to EU labelling at first
        let mut eu_labelling = until_labelling.clone();

        // choose and process states from working set until empty
        while let Some(state_id) = working.pop_front() {
            let state_eu_labelling = *eu_labelling
                .get(&state_id)
                .expect("EU labelling should contain working state");

            // for every parent of the chosen state which is labelled (f) but not EU(f,g) yet,
            // try labelling true or unknown and, if it changes things, add to the working set
            for previous_id in self.space.direct_predecessor_iter(state_id.into()) {
                // ignore start node
                let Ok(previous_id) = StateId::try_from(previous_id) else {
                    continue;
                };

                let previous_hold_labelling = hold_labelling
                    .get(&previous_id)
                    .expect("Hold labelling should contain previous state");
                let previous_eu_labelling = eu_labelling
                    .get_mut(&previous_id)
                    .expect("EU labelling should contain previous state");
                let val = *previous_eu_labelling;
                *previous_eu_labelling =
                    *previous_eu_labelling | (*previous_hold_labelling & state_eu_labelling);
                if *previous_eu_labelling != val {
                    working.push_back(previous_id);
                }
            }
        }

        Ok(eu_labelling)
    }

    fn constant_labelling(&self, value: ThreeValued) -> BTreeMap<StateId, ThreeValued> {
        BTreeMap::from_iter(self.space.nodes().filter_map(|node_id| {
            let Ok(state_id) = StateId::try_from(node_id) else {
                return None;
            };
            Some((state_id, value))
        }))
    }
}
