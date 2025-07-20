use std::collections::{BTreeMap, BTreeSet};

use log::{log_enabled, trace};
use machine_check_common::{property::FixedPointOperator, ExecError, StateId, ThreeValued};

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

            let history = {
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

        self.compute_labelling(params.op.inner)?;

        params.time_instant += 1;

        //println!("Updated in this iteration: {:?}", current_update);

        let inner_computation = Self::computation(&self.computations, params.op.inner);

        for state_id in inner_computation.updated.iter().copied() {
            let fixed_point_value = self.value_opt(params.subproperty_index, state_id);
            let inner_value = inner_computation.values.get(&state_id).unwrap();
            if fixed_point_value != Some(inner_value) {
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
        let our_computation = Self::computation(&self.computations, subproperty_index);
        let fixed_point_computation = Self::computation(&self.computations, fixed_point_index);

        let mut update = BTreeMap::new();

        for state_id in our_computation
            .updated
            .union(&fixed_point_computation.updated)
            .cloned()
        {
            let fixed_point_value = self.fixed_point_value(fixed_point_index, state_id);
            let valuation = fixed_point_value.valuation;
            // drop the next states

            update.insert(state_id, CheckValue::eigen(valuation));
        }

        self.update_subproperty(subproperty_index, update);
        Ok(())
    }

    pub fn fixed_point_value(&self, subproperty_index: usize, state_id: StateId) -> &CheckValue {
        let value = self.value_opt(subproperty_index, state_id);
        if let Some(value) = value {
            return value;
        }

        // TODO: use the correct subproperty and timing

        panic!();
        /*let old_cache_entry = self
            .property_checker
            .old_cache
            .get(self.property_checker.old_cache_index)
            .expect("Value computation should have old cache entry");

        assert_eq!(subproperty_index, old_cache_entry.fixed_point_index);

        let Some(old_state) = old_cache_entry.history.states.get(&state_id) else {
            panic!(
                "Value computation should have old values for state {}",
                state_id
            );
        };

        // TODO: get the value
        let (_time, value) = old_state
            .last_key_value()
            .expect("Value computation should have last old state value");
        value*/
    }
}
