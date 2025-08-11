use std::{collections::BTreeMap, ops::ControlFlow};

use log::trace;
use machine_check_common::{property::FixedPointOperator, ExecError, ThreeValued};

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
}

impl<M: FullMachine> LabellingUpdater<'_, M> {
    pub fn update_fixed_point_op(
        &mut self,
        fixed_point_index: usize,
        op: &FixedPointOperator,
    ) -> Result<(), ExecError> {
        if self.invalidate {
            // just invalidate fast
            return Ok(());
        }

        let start_time = self.current_time;

        // either check old computation (excluding end time) or add new computation

        let current_computation_index = self.next_computation_index;
        self.next_computation_index += 1;

        let old_computation_end_time = match self.process_old_end_time(
            fixed_point_index,
            current_computation_index,
            start_time,
        )? {
            ControlFlow::Break(()) => return Ok(()),
            ControlFlow::Continue(old_computation_end_time) => old_computation_end_time,
        };

        // test for calmness, the fixed point must be in closed form
        // and also already once computed
        if self.process_calm(fixed_point_index, current_computation_index, start_time)? {
            return Ok(());
        }

        trace!(
            "Fixed point index {}, current computation index {}, start time {}",
            fixed_point_index,
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
        };

        while let ControlFlow::Continue(()) = self.fixed_point_iteration(&mut params)? {}

        // we reached the fixed point
        // the inner updated have been cleared

        trace!("Fixed point {:?} reached", params.fixed_point_index);

        let computation_clone =
            self.property_checker.computations[current_computation_index].clone();

        if old_computation_end_time.is_some() {
            self.adjust_end_time_using_old(start_time, computation_clone)?;
        } else {
            self.adjust_end_time_padding(current_computation_index, computation_clone);
        }

        // since this fixed point was computed properly, it can be considered when calming afterwards
        self.calmable_fixed_points.insert(fixed_point_index);

        // select the states to propagate
        /*let history = select_history(&mut self.property_checker.histories, fixed_point_index);
        let mut result = BTreeMap::new();
        for state_id in self.property_checker.focus.affected().iter().copied() {
            let start_timed = history.opt_before_time(start_time, state_id);
            let end_timed = history.opt_before_time(self.current_time, state_id);
            let changed = if let (Some(start_timed), Some(end_timed)) = (start_timed, end_timed) {
                start_timed.value != end_timed.value
            } else {
                true
            };
            if changed {
                result.insert(state_id, history.before_time(self.current_time, state_id));
            }
        }
        trace!(
            "End computation index {}, next computation index {}, length {}",
            current_computation_index,
            self.next_computation_index,
            self.property_checker.computations.len()
        );*/

        Ok(())
    }

    fn fixed_point_iteration(
        &mut self,
        params: &mut FixedPointIterationParams,
    ) -> Result<ControlFlow<(), ()>, ExecError> {
        trace!("Fixed point {:?} not reached yet", params.fixed_point_index);
        trace!("Histories: {:?}", self.property_checker.histories);

        // increment time
        self.current_time += 1;
        // TODO: update the cache properly
        self.property_checker.latest_cache.get_mut().clear_all();

        // compute the iteration

        self.update_labelling(params.inner_index)?;

        // TODO: compute the current update properly
        let mut current_update = BTreeMap::new();
        for state_id in self.property_checker.focus.affected().iter().copied() {
            let value = self
                .property_checker
                .get_cached(params.inner_index, state_id);
            current_update.insert(state_id, value);
        }

        trace!("Current update: {:?}", current_update);

        let history = select_history(
            &mut self.property_checker.histories,
            params.fixed_point_index,
        );

        for (state_id, update_timed) in current_update {
            // check if the update differs

            let old_timed = history.before_time(self.current_time, state_id);

            if update_timed.value.valuation == old_timed.value.valuation {
                continue;
            }

            // the update differs
            // insert the state and make it dirty
            history.insert(self.current_time, state_id, update_timed.value);
            self.property_checker
                .focus
                .insert_dirty(self.space, state_id);
        }

        if !history.time_changes(self.current_time) {
            return Ok(ControlFlow::Break(()));
        }

        Ok(ControlFlow::Continue(()))
    }

    pub fn update_fixed_variable(&mut self, fixed_point_index: usize) -> Result<(), ExecError> {
        // recache the values of affected, not just dirty
        for state_id in self.property_checker.focus.affected() {
            self.getter().force_recache(fixed_point_index, *state_id)?;
        }
        Ok(())
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
