use std::{
    collections::{BTreeMap, BTreeSet},
    ops::ControlFlow,
};

use log::trace;
use machine_check_common::{property::FixedPointOperator, ExecError, StateId, ThreeValued};

use crate::{
    model_check::property_checker::{labelling_computer::LabellingComputer, CheckValue},
    FullMachine,
};

struct FixedPointIterationParams<'a> {
    fixed_point_index: usize,
    op: &'a FixedPointOperator,
}

impl<M: FullMachine> LabellingComputer<'_, M> {
    pub fn compute_fixed_point_op(
        &mut self,
        subproperty_index: usize,
        op: &FixedPointOperator,
    ) -> Result<BTreeSet<StateId>, ExecError> {
        if self.is_calm(subproperty_index, &mut Vec::new()) {
            trace!(
                "Not computing fixed point {} as it is calm",
                subproperty_index
            );
            return Ok(BTreeSet::new());
        }

        // clear updated as it will be regenerated

        /*let fixed_point_update = self
            .updates
            .get_mut(&subproperty_index)
            .expect("Fixed point should have an update");
        fixed_point_update.clear();*/

        // update history to ground values

        let ground_value = CheckValue::eigen(ThreeValued::from_bool(op.is_greatest));

        let mut params = FixedPointIterationParams {
            fixed_point_index: subproperty_index,
            op,
        };

        for state_id in self.space.states() {
            self.fixed_point_update(&params, state_id, ground_value.clone());
        }

        /*let dirty = BTreeSet::from_iter(self.space.states());
        self.propagate_updates(op.inner, &dirty);*/

        // compute inner property labelling and update variable labelling until the fixpoint is reached
        while let ControlFlow::Continue(()) = self.fixed_point_iteration(&mut params)? {}

        self.fixed_point_conclusion(params)?;

        let updated = BTreeSet::from_iter(self.space.states());

        Ok(updated)
    }

    fn fixed_point_iteration(
        &mut self,
        params: &mut FixedPointIterationParams,
    ) -> Result<ControlFlow<(), ()>, ExecError> {
        /*let fixed_point_update = self
        .computations
        .get(&params.fixed_point_index)
        .expect("Fixed point update should be available");*/

        trace!("Fixed point {:?} not reached yet", params.fixed_point_index);

        trace!("Fixed point values: {:?}", self.fixed_point_values);

        let current_update = self.compute_labelling(params.op.inner)?;

        trace!(
            "Latest after computing subproperties of fixed point {:?}: {:#?}",
            params.fixed_point_index,
            self.property_checker.latest
        );

        //println!("Updated in this iteration: {:?}", current_update);

        let mut result = ControlFlow::Break(());

        /*let inner_updated = self
        .updates
        .get_mut(&params.op.inner)
        .expect("Updates should contain fixed-point inner");

        let mut current_update = BTreeSet::new();

        std::mem::swap(inner_updated, &mut current_update);

        let current_update = self.space.states();*/

        for state_id in current_update {
            let inner_labelling = self.property_checker.get_labelling(params.op.inner);

            let inner_value = inner_labelling
                .get(&state_id)
                .expect("Fixed-point inner labelling should contain updated state")
                .clone();
            let fixed_point_values = self
                .fixed_point_values
                .get(&params.fixed_point_index)
                .expect("Values should be available for the current fixed point");
            let fixed_point_value = fixed_point_values
                .get(&state_id)
                .expect("Current fixed point value should be available");

            self.value_opt(params.fixed_point_index, state_id);

            if *fixed_point_value == inner_value.valuation {
                // nothing to do here
                continue;
            }

            if let ControlFlow::Break(()) = result {
                // fixed point not yet reached
                // increment time instant
                result = ControlFlow::Continue(());
                self.time_instant += 1;
            }

            self.fixed_point_update(params, state_id, inner_value.clone());

            /*self.property_checker
            .latest
            .get_mut(&params.fixed_point_index)
            .expect("Latest should contain fixed-point property")
            .insert(state_id, inner_value);*/
        }

        *self
            .property_checker
            .latest
            .get_mut(&params.fixed_point_index)
            .expect("Latest should contain fixed-point property") =
            self.property_checker.get_labelling(params.op.inner).clone();

        //let dirty = BTreeSet::from_iter(self.space.states());
        //self.propagate_updates(params.op.inner, &dirty);

        Ok(result)
    }

    fn fixed_point_update(
        &mut self,
        params: &FixedPointIterationParams,
        state_id: StateId,
        new_value: CheckValue,
    ) {
        // we have to modify the update, latest, and history

        /*self.updates
        .get_mut(&params.fixed_point_index)
        .expect("Updates should contain fixed-point property")
        .insert(state_id);*/

        /*self.property_checker
        .latest
        .get_mut(&params.fixed_point_index)
        .expect("Latest should contain fixed-point property")
        .insert(state_id, new_value.clone());*/

        self.fixed_point_values
            .entry(params.fixed_point_index)
            .or_default()
            .insert(state_id, new_value.valuation);

        self.property_checker
            .fixed_point_histories
            .get_mut(&params.fixed_point_index)
            .expect("Fixed point histories should contain property")
            .insert_update(self.time_instant, state_id, new_value);
    }

    fn fixed_point_conclusion(
        &mut self,
        params: FixedPointIterationParams,
    ) -> Result<(), ExecError> {
        // we reached the fixed point
        // the inner updated have been cleared

        trace!("Fixed point {:?} reached", params.fixed_point_index);

        Ok(())
    }

    pub fn compute_fixed_variable(
        &mut self,
        subproperty_index: usize,
        fixed_point_index: usize,
    ) -> Result<BTreeSet<StateId>, ExecError> {
        let mut update = BTreeMap::new();

        //let dirty = Self::computation(&self.updates, subproperty_index).iter().copied();
        let dirty = self.space.states();

        for state_id in dirty {
            //let fixed_point_value = self.fixed_point_value(fixed_point_index, state_id);
            //let valuation = fixed_point_value.valuation;
            // drop the next states

            let fixed_point_values = self
                .fixed_point_values
                .get(&fixed_point_index)
                .expect("Fixed point subproperty for variable should have values");

            let valuation = fixed_point_values
                .get(&state_id)
                .expect("Fixed point values should contain state");

            update.insert(state_id, CheckValue::eigen(*valuation));
        }

        self.update_subproperty(subproperty_index, update)?;

        Ok(BTreeSet::from_iter(self.space.states()))
    }

    /*pub fn fixed_point_value(&self, subproperty_index: usize, state_id: StateId) -> &CheckValue {
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
    }*/
}
