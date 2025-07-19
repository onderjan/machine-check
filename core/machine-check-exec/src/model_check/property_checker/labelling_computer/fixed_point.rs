use std::collections::{BTreeMap, BTreeSet};

use log::{log_enabled, trace};
use machine_check_common::{
    property::{FixedPointOperator, PropertyType},
    ExecError, StateId, ThreeValued,
};

use crate::{
    model_check::property_checker::{
        labelling_computer::LabellingComputer, CacheEntry, CheckValue, FixedPointHistory,
    },
    FullMachine,
};

struct FixedPointIterationParams<'a> {
    subproperty_index: usize,
    op: &'a FixedPointOperator,

    time_instant: u64,

    history: FixedPointHistory,
    old_values: BTreeMap<StateId, CheckValue>,
    update: BTreeMap<StateId, CheckValue>,
    all_updated: BTreeSet<StateId>,
}

impl<M: FullMachine> LabellingComputer<'_, M> {
    pub fn compute_fixed_point_op(
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

        while let Some(old_cache_entry) = self
            .property_checker
            .old_cache
            .get(self.property_checker.old_cache_index)
        {
            if old_cache_entry.fixed_point_index == subproperty_index {
                break;
            }
            self.property_checker.old_cache_index += 1;
        }

        // TODO: do not clone old cache entry
        let history = self
            .property_checker
            .old_cache
            .get(self.property_checker.old_cache_index)
            .map(|entry| entry.history.clone());

        let ground_value = CheckValue::eigen(ThreeValued::from_bool(op.is_greatest));

        // initialise fixed-point computation labelling

        let (old_values, update, history) = {
            let fixed_point_computation = self
                .computations
                .get_mut(&subproperty_index)
                .expect("Fixed-point operation should have a computation");

            let mut old_values = BTreeMap::new();
            std::mem::swap(&mut old_values, &mut fixed_point_computation.values);

            fixed_point_computation.updated.clear();

            let mut update = BTreeMap::new();

            let history = if let Some(history) = history {
                for state_id in self.property_checker.purge_states.iter().copied() {
                    update.insert(state_id, ground_value.clone());
                }
                history
            } else {
                for state_id in self.space.states() {
                    update.insert(state_id, ground_value.clone());
                }
                FixedPointHistory {
                    times: BTreeMap::new(),
                    states: BTreeMap::new(),
                }
            };

            trace!(
                "Starting fixed-point {:?} computation, current: {:?}, old values: {:?}",
                subproperty_index,
                fixed_point_computation,
                old_values,
            );
            (old_values, update, history)
        };

        let mut params = FixedPointIterationParams {
            subproperty_index,
            op,

            time_instant: 0,

            history,
            old_values,
            update,
            all_updated: BTreeSet::new(),
        };

        // compute inner property labelling and update variable labelling until they match
        while !params.update.is_empty() {
            self.fixed_point_iteration(&mut params)?;
        }

        self.fixed_point_conclusion(params)?;

        Ok(())
    }

    fn fixed_point_iteration(
        &mut self,
        params: &mut FixedPointIterationParams,
    ) -> Result<(), ExecError> {
        trace!(
            "Fixed point {:?} not reached yet, update: {:?}",
            params.subproperty_index,
            params.update
        );

        // fixed point not reached yet
        // change the values and updated

        let mut current_update = BTreeMap::new();
        std::mem::swap(&mut params.update, &mut current_update);

        let partial_updates = BTreeSet::from_iter(current_update.keys().copied());
        self.propagate_updates(params.subproperty_index, &partial_updates);

        let fixed_point_computation = self
            .computations
            .get_mut(&params.subproperty_index)
            .expect("Fixed-point operation should have a computation");

        for (state_id, update_value) in current_update {
            fixed_point_computation
                .values
                .insert(state_id, update_value.clone());
            fixed_point_computation.updated.insert(state_id);
            params.all_updated.insert(state_id);
            params
                .history
                .insert_update(params.time_instant, state_id, update_value);
        }

        trace!(
            "Fixed point {:?} updated: {:?}",
            params.subproperty_index,
            fixed_point_computation
        );

        if log_enabled!(log::Level::Trace) {
            for state_id in self.space.states() {
                let current = fixed_point_computation.values.get(&state_id);
                let old = params
                    .history
                    .states
                    .get(&state_id)
                    .and_then(|state_history| {
                        state_history
                            .range(0..=params.time_instant)
                            .last()
                            .map(|(_, value)| value)
                    });
                if current != old {
                    trace!(
                        "State {} mismatch from old {:?} to current {:?}",
                        state_id,
                        old,
                        current
                    );
                }
            }
        }

        self.compute_labelling(params.op.inner)?;

        params.time_instant += 1;

        //println!("Updated in this iteration: {:?}", current_update);

        let inner_computation = Self::computation(&self.computations, params.op.inner);

        for state_id in inner_computation.updated.iter().copied() {
            let fixed_point_value = self.value(params.subproperty_index, state_id);
            let inner_value = inner_computation.values.get(&state_id).unwrap();
            if fixed_point_value != inner_value {
                params.update.insert(state_id, inner_value.clone());
            }
        }

        let inner_computation = Self::computation_mut(&mut self.computations, params.op.inner);
        inner_computation.updated.clear();

        Ok(())
    }

    fn fixed_point_conclusion(
        &mut self,
        params: FixedPointIterationParams,
    ) -> Result<(), ExecError> {
        // we reached the fixed point
        // the updated values from the outside point of view are those that differ from the ones before
        // these must have been updated at least once

        let fixed_point_computation =
            Self::computation_mut(&mut self.computations, params.subproperty_index);

        fixed_point_computation.updated.clear();

        for state_id in params.all_updated.iter().cloned() {
            let current_value = fixed_point_computation.values.get(&state_id).unwrap();
            if let Some(old_value) = params.old_values.get(&state_id) {
                if old_value == current_value {
                    continue;
                }
            }
            fixed_point_computation.updated.insert(state_id);
        }

        self.property_checker.cache.push(CacheEntry {
            fixed_point_index: params.subproperty_index,
            history: params.history,
        });

        trace!(
            "Fixed point {:?} reached: {:?}",
            params.subproperty_index,
            fixed_point_computation
        );

        Ok(())
    }

    pub fn compute_fixed_variable(
        &mut self,
        subproperty_index: usize,
        fixed_point_index: usize,
    ) -> Result<(), ExecError> {
        // everything is given by the fixed point
        let fixed_point_computation = Self::computation(&self.computations, fixed_point_index);

        let mut update = BTreeMap::new();

        for state_id in fixed_point_computation.updated.iter().cloned() {
            let fixed_point_value = self.value(fixed_point_index, state_id);
            // drop the next states

            update.insert(state_id, CheckValue::eigen(fixed_point_value.valuation));
        }

        self.update_subproperty(subproperty_index, update);
        Ok(())
    }

    fn propagate_updates(&mut self, subproperty_index: usize, partial_updates: &BTreeSet<StateId>) {
        let subproperty_entry = self.property.subproperty_entry(subproperty_index);

        trace!(
            "Propagating down to subproperty {} with partial updates {:?}",
            subproperty_index,
            partial_updates
        );
        let computation = Self::computation_mut(&mut self.computations, subproperty_index);
        computation.updated.extend(partial_updates);

        let mut add_updates = BTreeSet::new();

        match &subproperty_entry.ty {
            PropertyType::Const(_) | PropertyType::Atomic(_) => {
                // nothing to propagate
            }
            PropertyType::Negation(inner) => {
                self.propagate_updates(*inner, partial_updates);
                let inner_computation = Self::computation(&self.computations, *inner);
                add_updates.extend(inner_computation.updated.iter().copied());
            }
            PropertyType::BiLogic(op) => {
                self.propagate_updates(op.a, partial_updates);
                self.propagate_updates(op.b, partial_updates);

                let a_computation = Self::computation(&self.computations, op.a);
                add_updates.extend(a_computation.updated.iter().copied());
                let b_computation = Self::computation(&self.computations, op.b);
                add_updates.extend(b_computation.updated.iter().copied());
            }
            PropertyType::Next(op) => {
                let mut next_partial_updates = BTreeSet::new();
                for state_id in partial_updates.iter().copied() {
                    for successor_id in self.space.direct_successor_iter(state_id.into()) {
                        next_partial_updates.insert(successor_id);
                    }
                }
                self.propagate_updates(op.inner, &next_partial_updates);

                let inner_computation = Self::computation(&self.computations, op.inner);
                for state_id in inner_computation.updated.iter().copied() {
                    for predecessor_id in self.space.direct_predecessor_iter(state_id.into()) {
                        if let Ok(predecessor_id) = StateId::try_from(predecessor_id) {
                            add_updates.insert(predecessor_id);
                        }
                    }
                }
            }
            PropertyType::FixedPoint(op) => {
                self.propagate_updates(op.inner, partial_updates);

                let inner_computation = Self::computation(&self.computations, op.inner);
                add_updates.extend(inner_computation.updated.iter().copied());
            }
            PropertyType::FixedVariable(fixed_point_index) => {
                // nothing to propagate
                let fixed_variable_computation =
                    Self::computation(&self.computations, *fixed_point_index);
                add_updates.extend(fixed_variable_computation.updated.iter().copied());
            }
        };

        let computation = Self::computation_mut(&mut self.computations, subproperty_index);
        computation.updated.extend(add_updates);
    }
}
