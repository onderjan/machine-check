use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet, VecDeque},
};

use log::{log_enabled, trace};
use machine_check_common::{
    check::Property,
    property::{BiLogicOperator, FixedPointOperator, NextOperator, PropertyType},
    ExecError, StateId, ThreeValued,
};
use mck::concr::FullMachine;

use crate::{
    model_check::property_checker::{CacheEntry, CheckValue, PropertyChecker},
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
        self.property_checker.cache.clear();

        self.compute_labelling(0)?;

        //self.very_dirty.clear();
        let values = &self.computations.get(&0).unwrap().values;
        // conventionally, the property must hold in all initial states
        let mut result = ThreeValued::True;
        for initial_state_id in self.space.initial_iter() {
            let value = values
                .get(&initial_state_id)
                .expect("Labelling should contain initial state");
            let state_value = value.valuation;

            result = result & state_value;
        }

        if log_enabled!(log::Level::Trace) {
            trace!("Computed interpretation of {:?}", self.property);

            /*for (subproperty_index, check_info) in &self.subproperty_map {
                let subproperty = property.subproperty_entry(*subproperty_index);

                let mut display = format!(
                    "Subproperty {} ({:?}): resets {:?}, labelling [\n",
                    subproperty_index, subproperty, check_info
                );
                for (state_id, label) in &check_info.labelling {
                    display.push_str(&format!("\t{}: {:?}\n", state_id, label));
                }
                display.push_str("]\n");

                trace!("{}", display);
            }*/
        }

        Ok(result)
    }

    fn compute_labelling(&mut self, subproperty_index: usize) -> Result<(), ExecError> {
        // take the frontier

        //println!("Property: {:?}", self.property);
        //println!("Computing labelling for index {}", subproperty_index);

        let subproperty_entry = self.property.subproperty_entry(subproperty_index);

        //println!("Computing labelling for {:?}", subproperty_entry);

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

    fn compute_negation(
        &mut self,
        subproperty_index: usize,
        inner: usize,
    ) -> Result<(), ExecError> {
        self.compute_labelling(inner)?;

        let inner_computation = Self::computation_mut(&mut self.computations, inner);

        // negate everything that was updated
        let mut update = BTreeMap::new();
        for state_id in &inner_computation.updated {
            let mut value = inner_computation.values.get(state_id).unwrap().clone();
            // negate
            value.valuation = !value.valuation;
            update.insert(*state_id, value);
        }

        inner_computation.updated.clear();

        self.update_subproperty(subproperty_index, update);
        Ok(())
    }

    fn compute_binary_op(
        &mut self,
        subproperty_index: usize,
        op: &BiLogicOperator,
    ) -> Result<(), ExecError> {
        self.compute_labelling(op.a)?;
        self.compute_labelling(op.b)?;

        let a_computation = Self::computation(&self.computations, op.a);
        let b_computation = Self::computation(&self.computations, op.b);

        let mut dirty = a_computation.updated.clone();
        dirty.extend(b_computation.updated.iter());

        let mut update = BTreeMap::new();

        for state_id in dirty {
            let a_value = a_computation
                .values
                .get(&state_id)
                .expect("Binary operation should have left value available");
            let b_value = b_computation
                .values
                .get(&state_id)
                .expect("Binary operation should have right value available");

            let a_valuation = a_value.valuation;
            let b_valuation = b_value.valuation;

            // TODO: freeze decision

            let result_value = if op.is_and {
                // we prefer the lesser value
                match a_valuation.cmp(&b_valuation) {
                    Ordering::Less => a_value,
                    Ordering::Equal => a_value,
                    Ordering::Greater => b_value,
                }
            } else {
                // we prefer the greater value
                match a_valuation.cmp(&b_valuation) {
                    Ordering::Less => b_value,
                    Ordering::Equal => a_value,
                    Ordering::Greater => a_value,
                }
            };

            update.insert(state_id, result_value.clone());
        }

        let a_computation = Self::computation_mut(&mut self.computations, op.a);
        a_computation.updated.clear();
        let b_computation = Self::computation_mut(&mut self.computations, op.b);
        b_computation.updated.clear();

        self.update_subproperty(subproperty_index, update);

        Ok(())
    }

    fn compute_next_labelling(
        &mut self,
        subproperty_index: usize,
        op: &NextOperator,
    ) -> Result<(), ExecError> {
        let ground_value = CheckValue::eigen(ThreeValued::from_bool(op.is_universal));
        self.compute_labelling(op.inner)?;

        let inner_computation = Self::computation(&self.computations, op.inner);

        // We need to compute states where the inner property was updated for their direct successors.

        let mut dirty = BTreeSet::new();

        for state_id in inner_computation.updated.iter().cloned() {
            for predecessor_id in self.space.direct_predecessor_iter(state_id.into()) {
                if let Ok(predecessor_id) = StateId::try_from(predecessor_id) {
                    dirty.insert(predecessor_id);
                }
            }
        }

        let mut update = BTreeMap::new();

        // For each state in dirty states, compute the new value from the successors.
        for dirty_id in dirty.iter().copied() {
            let old_successor = Self::computation(&self.computations, subproperty_index)
                .values
                .get(&dirty_id)
                .and_then(|value| value.next_states.last().copied());

            let mut direct_successors = VecDeque::new();
            for successor_id in self.space.direct_successor_iter(dirty_id.into()) {
                let value = inner_computation
                    .values
                    .get(&successor_id)
                    .expect("Direct successor should have values")
                    .clone();
                if old_successor.is_some_and(|old_successor| old_successor == successor_id) {
                    direct_successors.push_front((successor_id, value));
                } else {
                    direct_successors.push_back((successor_id, value));
                }
            }

            let mut current_value = ground_value.clone();

            for (successor_id, successor_value) in &direct_successors {
                let new_valuation = if op.is_universal {
                    current_value.valuation & successor_value.valuation
                } else {
                    current_value.valuation | successor_value.valuation
                };

                if current_value.valuation != new_valuation {
                    current_value.valuation = new_valuation;
                    current_value.next_states = successor_value.next_states.clone();
                    current_value.next_states.push(*successor_id);
                }
            }

            update.insert(dirty_id, current_value);
        }

        let inner_computation = Self::computation_mut(&mut self.computations, op.inner);
        inner_computation.updated.clear();

        self.update_subproperty(subproperty_index, update);

        //println!("Next valuations: {:?}", update);
        //println!("Next reasons: {:?}", reasons);

        Ok(())
    }

    fn compute_fixed_point_op(
        &mut self,
        subproperty_index: usize,
        op: &FixedPointOperator,
    ) -> Result<(), ExecError> {
        if self.is_calm(subproperty_index, &mut Vec::new()) {
            trace!(
                "Not computing fixed point {} as it is calm",
                subproperty_index
            );
            return Ok(());
        }

        self.property_checker.cache.push(CacheEntry {
            fixed_point_index: subproperty_index,
            time_instant: 0,
            histories: BTreeMap::new(),
        });

        let ground_value = CheckValue::eigen(ThreeValued::from_bool(op.is_greatest));

        // initialise fixed-point computation labelling

        let (old_values, mut update) = {
            let fixed_point_computation = self
                .computations
                .get_mut(&subproperty_index)
                .expect("Fixed-point operation should have a computation");

            let mut old_values = BTreeMap::new();
            std::mem::swap(&mut old_values, &mut fixed_point_computation.values);

            fixed_point_computation.updated.clear();

            let mut update = BTreeMap::new();

            for state_id in self.space.states() {
                update.insert(state_id, ground_value.clone());
            }

            trace!(
                "Starting fixed-point {:?} computation, current: {:?}, old values: {:?}",
                subproperty_index,
                fixed_point_computation,
                old_values,
            );
            (old_values, update)
        };

        let mut all_updated = BTreeSet::new();

        // compute inner property labelling and update variable labelling until they match
        loop {
            trace!(
                "Fixed point {:?} not reached yet, update: {:?}",
                subproperty_index,
                update
            );

            // fixed point not reached yet
            // change the values and updated

            let mut current_update = BTreeMap::new();
            std::mem::swap(&mut update, &mut current_update);

            {
                let cache_entry = self.property_checker.cache.last_mut().unwrap();
                let fixed_point_computation = self
                    .computations
                    .get_mut(&subproperty_index)
                    .expect("Fixed-point operation should have a computation");

                for (state_id, update_value) in current_update {
                    fixed_point_computation
                        .values
                        .insert(state_id, update_value.clone());
                    fixed_point_computation.updated.insert(state_id);
                    all_updated.insert(state_id);
                    Self::insert_history(cache_entry, state_id, update_value);
                }

                trace!(
                    "Fixed point {:?} updated: {:?}",
                    subproperty_index,
                    fixed_point_computation
                );
            }

            self.compute_labelling(op.inner)?;

            self.property_checker.cache.last_mut().unwrap().time_instant += 1;

            //println!("Updated in this iteration: {:?}", current_update);

            let fixed_point_computation = Self::computation(&self.computations, subproperty_index);
            let inner_computation = Self::computation(&self.computations, op.inner);

            for state_id in inner_computation.updated.iter().copied() {
                let fixed_point_value = fixed_point_computation.values.get(&state_id).unwrap();
                let inner_value = inner_computation.values.get(&state_id).unwrap();
                if fixed_point_value != inner_value {
                    update.insert(state_id, inner_value.clone());
                }
            }

            let inner_computation = Self::computation_mut(&mut self.computations, op.inner);
            inner_computation.updated.clear();

            if update.is_empty() {
                // we reached the fixed point
                // the updated values from the outside point of view are those that differ from the ones before
                // these must have been updated at least once

                let fixed_point_computation =
                    Self::computation_mut(&mut self.computations, subproperty_index);

                fixed_point_computation.updated.clear();

                for state_id in all_updated {
                    let current_value = fixed_point_computation.values.get(&state_id).unwrap();
                    if let Some(old_value) = old_values.get(&state_id) {
                        if old_value == current_value {
                            continue;
                        }
                    }
                    fixed_point_computation.updated.insert(state_id);
                }

                trace!(
                    "Fixed point {:?} reached: {:?}",
                    subproperty_index,
                    fixed_point_computation
                );

                return Ok(());
            }
        }
    }

    fn insert_history(cache_entry: &mut CacheEntry, state_id: StateId, value: CheckValue) {
        let history = cache_entry
            .histories
            .entry(cache_entry.fixed_point_index)
            .or_default();

        history
            .points
            .entry(cache_entry.time_instant)
            .or_default()
            .insert(state_id, value.clone());

        history
            .states
            .entry(state_id)
            .or_default()
            .insert(cache_entry.time_instant, value);
    }

    fn compute_fixed_variable(
        &mut self,
        subproperty_index: usize,
        fixed_point_index: usize,
    ) -> Result<(), ExecError> {
        // everything is given by the fixed point

        let fixed_point_computation = Self::computation(&self.computations, fixed_point_index);

        let mut update = BTreeMap::new();

        for state_id in fixed_point_computation.updated.iter().cloned() {
            let fixed_point_value = fixed_point_computation.values.get(&state_id).unwrap();
            // drop the next states

            update.insert(state_id, CheckValue::eigen(fixed_point_value.valuation));
        }

        self.update_subproperty(subproperty_index, update);
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
                if *current_value == update_value {
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

    pub fn subproperty_values(&self, subproperty_index: usize) -> &BTreeMap<StateId, CheckValue> {
        &Self::computation(&self.computations, subproperty_index).values
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
}
