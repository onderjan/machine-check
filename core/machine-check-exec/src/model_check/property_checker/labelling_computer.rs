mod fixed_point;
mod local;
mod next;
mod propagate_updates;

use std::collections::{BTreeMap, BTreeSet};

use log::{log_enabled, trace};
use machine_check_common::{
    check::Property, property::PropertyType, ExecError, StateId, ThreeValued,
};
use mck::concr::FullMachine;

use crate::{
    model_check::property_checker::{CheckValue, PropertyChecker},
    space::StateSpace,
};

pub struct LabellingComputer<'a, M: FullMachine> {
    property_checker: &'a mut PropertyChecker,
    property: &'a Property,
    space: &'a StateSpace<M>,

    computations: BTreeMap<usize, SubpropertyComputation>,
}

#[derive(Debug)]
pub struct SubpropertyComputation {
    values: BTreeMap<StateId, CheckValue>,
    updated: BTreeSet<StateId>,
}

impl<'a, M: FullMachine> LabellingComputer<'a, M> {
    pub fn new(
        property_checker: &'a mut PropertyChecker,
        property: &'a Property,
        space: &'a StateSpace<M>,
    ) -> Result<Self, ExecError> {
        let mut computations = BTreeMap::new();
        for subproperty_index in 0..property.num_subproperties() {
            let subproperty = property.subproperty_entry(subproperty_index);
            let values = match &subproperty.ty {
                PropertyType::Const(constant) => {
                    let constant = ThreeValued::from_bool(*constant);
                    let eigen = CheckValue::eigen(constant);
                    let values = BTreeMap::from_iter(
                        space.states().map(|state_id| (state_id, eigen.clone())),
                    );
                    Some(values)
                }
                PropertyType::Atomic(atomic_property) => {
                    let mut values = BTreeMap::new();
                    for state_id in space.states() {
                        let value = space.atomic_label(atomic_property, state_id)?;
                        let value = CheckValue::eigen(value);
                        values.insert(state_id, value);
                    }
                    Some(values)
                }
                _ => None,
            };

            let computation = if let Some(values) = values {
                SubpropertyComputation {
                    values,
                    updated: BTreeSet::from_iter(space.states()),
                }
            } else {
                SubpropertyComputation {
                    values: BTreeMap::new(),
                    updated: BTreeSet::new(),
                }
            };

            computations.insert(subproperty_index, computation);
        }
        Ok(Self {
            property_checker,
            property,
            space,
            computations,
        })
    }

    pub fn compute(&mut self) -> Result<ThreeValued, ExecError> {
        self.property_checker.old_cache.clear();
        self.property_checker
            .old_cache
            .append(&mut self.property_checker.cache);
        self.property_checker.old_cache_index = 0;

        self.compute_labelling(0)?;

        self.property_checker.purge_states.clear();

        // conventionally, the property must hold in all initial states
        let mut result = ThreeValued::True;
        for initial_state_id in self.space.initial_iter() {
            let value = self.value(0, initial_state_id);
            let valuation = value.valuation;

            result = result & valuation;
        }

        if log_enabled!(log::Level::Trace) {
            trace!("Computed interpretation of {:?}", self.property);
        }

        Ok(result)
    }

    fn compute_labelling(&mut self, subproperty_index: usize) -> Result<(), ExecError> {
        let subproperty_entry = self.property.subproperty_entry(subproperty_index);

        match &subproperty_entry.ty {
            PropertyType::Const(_) | PropertyType::Atomic(_) => {
                // already precomputed
                return Ok(());
            }
            PropertyType::Negation(inner) => {
                self.compute_negation(subproperty_index, *inner)?;
            }
            PropertyType::BiLogic(op) => {
                self.compute_binary_op(subproperty_index, op)?;
            }
            PropertyType::Next(op) => {
                self.compute_next_labelling(subproperty_index, op)?;
            }
            PropertyType::FixedPoint(op) => {
                self.compute_fixed_point_op(subproperty_index, op)?;
            }
            PropertyType::FixedVariable(fixed_point_index) => {
                self.compute_fixed_variable(subproperty_index, *fixed_point_index)?;
            }
        };

        Ok(())
    }

    fn update_subproperty(
        &mut self,
        subproperty_index: usize,
        update: BTreeMap<StateId, CheckValue>,
    ) {
        let computation = self
            .computations
            .get_mut(&subproperty_index)
            .expect("Fixed-point operation should have a computation");

        if log_enabled!(log::Level::Trace) {
            let subproperty_entry = self.property.subproperty_entry(subproperty_index);
            trace!(
                "Updating subproperty {:?} ({:?}) with {:#?}",
                subproperty_index,
                subproperty_entry,
                update
            );
        }

        computation.updated.clear();

        for (state_id, update_value) in update {
            if let Some(current_value) = computation.values.get_mut(&state_id) {
                // do not update when the valuation is not changed
                if current_value.valuation == update_value.valuation {
                    continue;
                }
                *current_value = update_value;
            } else {
                computation.values.insert(state_id, update_value);
            }
            computation.updated.insert(state_id);
        }

        trace!("Updated {:?}", computation.updated);
    }

    fn computation(
        computations: &BTreeMap<usize, SubpropertyComputation>,
        index: usize,
    ) -> &SubpropertyComputation {
        computations.get(&index).expect("Computation should exist")
    }

    fn computation_mut(
        computations: &mut BTreeMap<usize, SubpropertyComputation>,
        index: usize,
    ) -> &mut SubpropertyComputation {
        computations
            .get_mut(&index)
            .expect("Computation should exist")
    }

    pub fn is_calm(&self, subproperty_index: usize, calm_fixed_points: &mut Vec<usize>) -> bool {
        let computation = Self::computation(&self.computations, subproperty_index);
        if !computation.updated.is_empty() {
            return false;
        }

        let subproperty_entry = self.property.subproperty_entry(subproperty_index);

        match &subproperty_entry.ty {
            PropertyType::Const(_) | PropertyType::Atomic(_) => true,
            PropertyType::Negation(inner) => self.is_calm(*inner, calm_fixed_points),
            PropertyType::BiLogic(bi_logic_operator) => {
                self.is_calm(bi_logic_operator.a, calm_fixed_points)
                    && self.is_calm(bi_logic_operator.b, calm_fixed_points)
            }
            PropertyType::Next(next_operator) => {
                self.is_calm(next_operator.inner, calm_fixed_points)
            }
            PropertyType::FixedPoint(fixed_point_operator) => {
                calm_fixed_points.push(subproperty_index);
                let result = self.is_calm(fixed_point_operator.inner, calm_fixed_points);
                calm_fixed_points.pop();
                result
            }
            PropertyType::FixedVariable(fixed_point_index) => {
                calm_fixed_points.contains(fixed_point_index)
            }
        }
    }

    pub fn value(&self, subproperty_index: usize, state_id: StateId) -> &CheckValue {
        let result = self.value_opt(subproperty_index, state_id);

        if let Some(result) = result {
            result
        } else {
            panic!(
                "Cannot fetch value state id {:?} from subproperty {}",
                state_id, subproperty_index
            );
        }
    }

    pub fn value_opt(&self, subproperty_index: usize, state_id: StateId) -> Option<&CheckValue> {
        let fixed_point_computation = self
            .computations
            .get(&subproperty_index)
            .expect("Fixed-point operation should have a computation");
        trace!(
            "Fetching value of state id {:?} from subproperty {} computation {:?}",
            state_id,
            subproperty_index,
            fixed_point_computation
        );
        if let Some(value) = fixed_point_computation.values.get(&state_id) {
            return Some(value);
        }

        None
    }
}
