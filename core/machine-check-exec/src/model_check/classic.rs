use std::collections::{BTreeMap, HashMap};

use log::{log_enabled, trace};
use machine_check_common::{
    property::{
        BiOperator, FixedPointOperator, FixedPointVariable, Property, TemporalOperator, UniOperator,
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
    labelling_map: HashMap<PropertyId, BTreeMap<StateId, ThreeValued>>,
    reasons_map: HashMap<PropertyId, BTreeMap<StateId, StateId>>,
    property_ids: HashMap<Property, PropertyId>,
    next_property_id: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct PropertyId(u64);

impl<'a, M: FullMachine> ClassicChecker<'a, M> {
    /// Classic two-valued model checker with chosen interpretation of unknown labellings.
    pub fn new(space: &'a StateSpace<M>) -> Self {
        ClassicChecker {
            space,
            labelling_map: HashMap::new(),
            reasons_map: HashMap::new(),
            property_ids: HashMap::new(),
            next_property_id: 0,
        }
    }

    pub fn compute_interpretation(&mut self, prop: &Property) -> Result<ThreeValued, ExecError> {
        let property_id = self.compute_labelling(prop)?;
        let labelling = self.get_labelling(property_id);
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

    pub fn get_labelling(&self, property_id: PropertyId) -> &BTreeMap<StateId, ThreeValued> {
        self.labelling_map
            .get(&property_id)
            .expect("Labelling should be present")
    }

    pub fn get_reasons(&self, property_id: PropertyId) -> &BTreeMap<StateId, StateId> {
        self.reasons_map
            .get(&property_id)
            .expect("Reasons should be present")
    }

    pub fn get_reasons_mut(&mut self, property_id: PropertyId) -> &mut BTreeMap<StateId, StateId> {
        self.reasons_map.entry(property_id).or_default()
    }

    pub fn compute_and_get_labelling(
        &mut self,
        prop: &Property,
    ) -> Result<&BTreeMap<StateId, ThreeValued>, ExecError> {
        let property_id = self.compute_labelling(prop)?;
        Ok(self.get_labelling(property_id))
    }

    fn compute_labelling(&mut self, prop: &Property) -> Result<PropertyId, ExecError> {
        let property_id = self.get_or_insert_property_id(prop);
        if self.labelling_map.contains_key(&property_id) {
            // already contained
            return Ok(property_id);
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
                let inner_property_id = self.compute_labelling(&inner.0)?;
                let mut result = self.get_labelling(inner_property_id).clone();
                for value in result.values_mut() {
                    *value = !*value;
                }
                result
            }
            Property::Or(BiOperator { a, b }) => {
                let a_id = self.compute_labelling(a)?;
                let b_id = self.compute_labelling(b)?;
                let a_labelling = self.get_labelling(a_id);
                let b_labelling = self.get_labelling(b_id);
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
                let a_id = self.compute_labelling(a)?;
                let b_id = self.compute_labelling(b)?;
                let a_labelling = self.get_labelling(a_id);
                let b_labelling = self.get_labelling(b_id);
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
                _ => {
                    panic!("expected {:?} to have only X temporal operator", prop);
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
                assert!(self.labelling_map.contains_key(&property_id));
                return Ok(property_id);
            }
            _ => panic!("expected {:?} to be minimized", prop),
        };

        if log_enabled!(log::Level::Trace) {
            trace!("computed property {} labelling {:?}", prop, labelling);
        }

        //println!("Computed property {} labelling {:?}", prop, labelling);
        //println!("Space: {:?}", self.space);
        // insert the labelling to labelling map for future reference
        self.labelling_map.insert(property_id, labelling);

        Ok(property_id)
    }

    fn compute_fixed_point(
        &mut self,
        operator: &FixedPointOperator,
        initial_value: ThreeValued,
    ) -> Result<BTreeMap<StateId, ThreeValued>, ExecError> {
        // initialise variable labelling
        let variable_property = Property::FixedPointVariable(operator.variable.clone());
        let variable_property_id = self.get_or_insert_property_id(&variable_property);
        self.labelling_map
            .insert(variable_property_id, self.constant_labelling(initial_value));

        // compute inner property labelling and update variable labelling until they match
        loop {
            let inner_property_id = self.compute_labelling(&operator.inner)?;
            let current_labelling = self.get_labelling(inner_property_id).clone();
            let previous_labelling = self.get_labelling(variable_property_id);
            if previous_labelling == &current_labelling {
                break Ok(current_labelling);
            }
            let variable = operator.variable.clone();
            self.clear_affected_labellings(&variable, &operator.inner);
            self.labelling_map
                .insert(variable_property_id, current_labelling.clone());
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
            let property_id = self.get_or_insert_property_id(prop);
            self.labelling_map.remove(&property_id);
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

        let property_id = self.get_or_insert_property_id(&prop);

        let mut reasons = self.get_reasons_mut(property_id).clone();

        let inner_property_id = self.compute_labelling(&inner.0)?;
        let inner_labelling = self.get_labelling(inner_property_id);
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

        *self.get_reasons_mut(property_id) = reasons;

        Ok(labelling)
    }

    fn constant_labelling(&self, value: ThreeValued) -> BTreeMap<StateId, ThreeValued> {
        BTreeMap::from_iter(self.space.nodes().filter_map(|node_id| {
            let Ok(state_id) = StateId::try_from(node_id) else {
                return None;
            };
            Some((state_id, value))
        }))
    }

    pub(super) fn get_property_id(&self, property: &Property) -> Option<PropertyId> {
        self.property_ids.get(property).cloned()
    }

    fn get_or_insert_property_id(&mut self, property: &Property) -> PropertyId {
        if let Some(id) = self.get_property_id(property) {
            return id;
        }

        let id = PropertyId(self.next_property_id);
        self.next_property_id = self
            .next_property_id
            .checked_add(1)
            .expect("Property id should not overflow");
        self.property_ids.insert(property.clone(), id);
        id
    }
}
