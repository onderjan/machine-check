use std::{
    collections::{BTreeMap, BTreeSet},
    ops::ControlFlow,
};

use log::{debug, trace};
use machine_check_common::{property::FixedPointOperator, ExecError, StateId, ThreeValued};

use crate::{
    model_check::property_checker::{
        history::FixedPointHistory, labelling_updater::LabellingUpdater, CheckValue,
    },
    FullMachine,
};

mod time_adjustment;

struct FixedPointIterationParams {
    fixed_point_index: usize,
    inner_index: usize,
    old_computation_end_time: Option<u64>,
}

impl<M: FullMachine> LabellingUpdater<'_, M> {
    pub fn update_fixed_point_op(
        &mut self,
        fixed_point_index: usize,
        op: &FixedPointOperator,
    ) -> Result<BTreeSet<StateId>, ExecError> {
        if self.invalidate {
            // just invalidate fast
            return Ok(BTreeSet::new());
        }

        self.property_checker.latest_cache.get_mut().clear_all();

        let start_time = self.current_time;

        // either check old computation (excluding end time) or add new computation

        let current_computation_index = self.next_computation_index;
        self.next_computation_index += 1;
        self.num_fixed_point_computations += 1;

        let old_computation_end_time = match self.process_old_end_time(
            fixed_point_index,
            current_computation_index,
            start_time,
        )? {
            ControlFlow::Break(()) => return Ok(BTreeSet::new()),
            ControlFlow::Continue(old_computation_end_time) => old_computation_end_time,
        };

        // test for calmness, the fixed point must be in closed form
        // and also already once computed
        if self.process_calm(fixed_point_index, current_computation_index, start_time)? {
            return Ok(BTreeSet::new());
        }

        debug!(
            "Computing fixed point {} with {}/{} states dirty (current computation index {}, start time {})",
            fixed_point_index,
            self.property_checker.focus.dirty().len(),
            self.space.num_states(),
            current_computation_index,
            start_time
        );

        // update the dirty states to ground values
        // note that if there was no old computation, all states in the state space have been made dirty

        let ground_value = CheckValue::eigen(ThreeValued::from_bool(op.is_greatest));
        let history = select_history(&mut self.property_checker.histories, fixed_point_index);
        trace!("Focus: {:?}", self.property_checker.focus);
        for state_id in self.property_checker.focus.dirty_iter() {
            history.insert(start_time, state_id, ground_value.clone());
        }

        // iterate until the fixed point is reached

        let mut params = FixedPointIterationParams {
            fixed_point_index,
            inner_index: op.inner,
            old_computation_end_time,
        };

        while let ControlFlow::Continue(()) = self.fixed_point_iteration(&mut params)? {}

        // we reached the fixed point
        // the inner updated have been cleared

        debug!(
            "Reached fixed point {} with {}/{} states dirty",
            fixed_point_index,
            self.property_checker.focus.dirty().len(),
            self.space.num_states(),
        );

        let computation_clone =
            self.property_checker.computations[current_computation_index].clone();

        if old_computation_end_time.is_some() {
            self.adjust_end_time_using_old(start_time, computation_clone)?;
        } else {
            self.adjust_end_time_padding(current_computation_index, computation_clone);
        }

        debug!(
            "Adjusted end time of fixed point {} with {}/{} states dirty",
            fixed_point_index,
            self.property_checker.focus.dirty().len(),
            self.space.num_states(),
        );

        // since this fixed point was computed properly, it can be considered when calming afterwards
        self.calmable_fixed_points.insert(fixed_point_index);

        // select the states to propagate
        let history = select_history(&mut self.property_checker.histories, fixed_point_index);
        let mut result = BTreeSet::new();
        for state_id in self.property_checker.focus.dirty_iter() {
            let state_history = history
                .for_state(state_id)
                .expect("Dirty state should have its history");

            let previous_update = state_history.range(0..start_time).next_back();
            let current_update = state_history.range(0..self.current_time).next_back();

            if previous_update != current_update {
                result.insert(state_id);
            }
        }

        Ok(result)
    }

    fn fixed_point_iteration(
        &mut self,
        params: &mut FixedPointIterationParams,
    ) -> Result<ControlFlow<(), ()>, ExecError> {
        // increment time
        self.current_time += 1;
        self.num_fixed_point_iterations += 1;

        // TODO: update the cache properly
        self.property_checker.latest_cache.get_mut().clear_all();

        // compute the iteration

        let mut updated = self.update_labelling(params.inner_index)?;

        // add previously updated

        let history = select_history(
            &mut self.property_checker.histories,
            params.fixed_point_index,
        );
        if let Some(previously_updated) = history.states_at_exact_time_opt(self.current_time) {
            // this also needs to be updated
            for state_id in previously_updated.keys().copied() {
                if self.space.contains_state(state_id) {
                    updated.insert(state_id);
                }
            }
        }

        // TODO: compute the current update properly
        let mut current_update = BTreeMap::new();
        for state_id in updated {
            self.getter()
                .cache_if_uncached(params.inner_index, state_id)?;

            let value = self
                .property_checker
                .get_cached(params.inner_index, state_id)
                .value;
            current_update.insert(state_id, value);
        }

        trace!(
            "Current update of fixed point {:?} in time {:?}: {:#?}",
            params.fixed_point_index,
            self.current_time,
            current_update
        );

        let history = select_history(
            &mut self.property_checker.histories,
            params.fixed_point_index,
        );

        for (state_id, update_value) in current_update {
            // check if the update differs

            let now_timed = history.up_to_time(self.current_time, state_id);

            if update_value == now_timed.value {
                continue;
            }

            /*trace!(
                "Inserting dirty state {}: now {:?}, update: {:?}",
                state_id,
                now_timed,
                update_value
            );*/

            // the update differs
            // insert the state and make it dirty
            history.insert(self.current_time, state_id, update_value);
            self.property_checker
                .focus
                .insert_dirty(self.space, state_id);
        }

        if !history.time_changes(self.current_time) {
            if let Some(old_computation_end_time) = params.old_computation_end_time {
                if self.current_time >= old_computation_end_time
                    || !history.range_changes(self.current_time, old_computation_end_time)
                {
                    return Ok(ControlFlow::Break(()));
                }
            } else {
                return Ok(ControlFlow::Break(()));
            }
        }

        Ok(ControlFlow::Continue(()))
    }

    pub fn update_fixed_variable(
        &mut self,
        fixed_point_index: usize,
    ) -> Result<BTreeSet<StateId>, ExecError> {
        // update the values of the states that have been changed in last time instant
        let mut update = BTreeSet::new();
        let history = select_history(&mut self.property_checker.histories, fixed_point_index);
        let Some(changed_states) = history.states_at_exact_time_opt(self.current_time - 1) else {
            return Ok(BTreeSet::new());
        };
        for state_id in changed_states.keys() {
            update.insert(*state_id);
        }

        Ok(update)
    }
}

fn select_history(
    histories: &mut BTreeMap<usize, FixedPointHistory>,
    fixed_point_index: usize,
) -> &mut FixedPointHistory {
    histories
        .get_mut(&fixed_point_index)
        .expect("Fixed point histories should contain property")
}
