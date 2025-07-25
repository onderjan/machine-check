mod fixed_point;
mod local;
mod next;
mod propagate_updates;

use std::collections::{BTreeMap, BTreeSet};

use log::{log_enabled, trace};
use machine_check_common::{property::PropertyType, ExecError, StateId, ThreeValued};
use mck::concr::FullMachine;

use crate::{
    model_check::property_checker::{CheckValue, PropertyChecker},
    space::StateSpace,
};

pub struct LabellingComputer<'a, M: FullMachine> {
    property_checker: &'a mut PropertyChecker,
    space: &'a StateSpace<M>,

    updates: BTreeMap<usize, BTreeSet<StateId>>,
    time_instant: u64,
}

impl<'a, M: FullMachine> LabellingComputer<'a, M> {
    pub fn new(
        property_checker: &'a mut PropertyChecker,
        space: &'a StateSpace<M>,
    ) -> Result<Self, ExecError> {
        property_checker.reinit_labellings()?;

        let mut updates = BTreeMap::new();

        for subproperty_index in 0..property_checker.property.num_subproperties() {
            let ty = &property_checker
                .property
                .subproperty_entry(subproperty_index)
                .ty;
            let update = if matches!(ty, PropertyType::Const(_) | PropertyType::Atomic(_)) {
                property_checker.recompute_states.clone()
            } else {
                BTreeSet::new()
            };

            updates.insert(subproperty_index, update);
        }

        let computer = Self {
            property_checker,
            space,
            updates,
            time_instant: 0,
        };

        Ok(computer)
    }

    pub fn compute(&mut self) -> Result<ThreeValued, ExecError> {
        self.time_instant = 0;

        self.compute_labelling(0)?;

        self.property_checker.recompute_states.clear();

        // conventionally, the property must hold in all initial states
        let mut result = ThreeValued::True;
        for initial_state_id in self.space.initial_iter() {
            let value = self.value(0, initial_state_id);
            let valuation = value.valuation;

            result = result & valuation;
        }

        if log_enabled!(log::Level::Trace) {
            trace!(
                "Computed interpretation of {:?}",
                self.property_checker.property
            );
        }

        Ok(result)
    }

    fn compute_labelling(&mut self, subproperty_index: usize) -> Result<(), ExecError> {
        let subproperty_entry = self
            .property_checker
            .property
            .subproperty_entry(subproperty_index);

        let ty = subproperty_entry.ty.clone();

        match &ty {
            PropertyType::Const(constant) => {
                let constant = ThreeValued::from_bool(*constant);
                let eigen = CheckValue::eigen(constant);
                let update = Self::computation(&self.updates, subproperty_index);

                let latest = self
                    .property_checker
                    .latest
                    .get_mut(&subproperty_index)
                    .expect("Latest should contain const subproperty");

                for state_id in update.iter().copied() {
                    latest.insert(state_id, eigen.clone());
                }
            }
            PropertyType::Atomic(atomic_property) => {
                let update = Self::computation(&self.updates, subproperty_index);

                let latest = self
                    .property_checker
                    .latest
                    .get_mut(&subproperty_index)
                    .expect("Latest should contain const subproperty");

                for state_id in update.iter().copied() {
                    let value = self.space.atomic_label(atomic_property, state_id)?;
                    let value = CheckValue::eigen(value);
                    latest.insert(state_id, value);
                }
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
        /*let computation = self
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

        computation.updated.clear();*/

        let latest = self.property_checker.get_labelling_mut(subproperty_index);

        for (state_id, update_value) in update {
            if let Some(current_value) = latest.get_mut(&state_id) {
                // do not update when the valuation is not changed
                if current_value.valuation == update_value.valuation {
                    continue;
                }
                *current_value = update_value;
            } else {
                latest.insert(state_id, update_value);
            }
            //computation.updated.insert(state_id);
        }

        //trace!("Updated {:?}", computation.updated);
    }

    fn computation(
        updates: &BTreeMap<usize, BTreeSet<StateId>>,
        subroperty_index: usize,
    ) -> &BTreeSet<StateId> {
        updates.get(&subroperty_index).expect("Update should exist")
    }

    fn computation_mut(
        updates: &mut BTreeMap<usize, BTreeSet<StateId>>,
        subproperty_index: usize,
    ) -> &mut BTreeSet<StateId> {
        updates
            .get_mut(&subproperty_index)
            .expect("Updare should exist")
    }

    pub fn is_calm(&self, subproperty_index: usize, calm_fixed_points: &mut Vec<usize>) -> bool {
        let update = Self::computation(&self.updates, subproperty_index);
        if !update.is_empty() {
            trace!(
                "Subproperty {} is not calm as it has updates",
                subproperty_index
            );
            return false;
        }

        let subproperty_entry = self
            .property_checker
            .property
            .subproperty_entry(subproperty_index);

        let result = match &subproperty_entry.ty {
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
        };

        trace!("Subproperty {} calmness: {}", subproperty_index, result);

        result
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
        let latest = self.property_checker.get_labelling(subproperty_index);

        latest.get(&state_id)
    }
}
