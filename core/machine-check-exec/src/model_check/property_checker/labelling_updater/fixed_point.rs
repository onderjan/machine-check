use std::{collections::BTreeMap, ops::ControlFlow};

use log::{debug, trace};
use machine_check_common::{property::FixedPointOperator, ExecError, ParamValuation, StateId};

use crate::{
    model_check::property_checker::{
        history::{FixedPointHistory, TimedCheckValue},
        labelling_updater::LabellingUpdater,
        CheckValue,
    },
    FullMachine,
};

mod iteration;
mod misc;
mod time_adjustment;
mod variable;

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
    ) -> Result<BTreeMap<StateId, TimedCheckValue>, ExecError> {
        if self.invalidate {
            // just invalidate fast
            return Ok(BTreeMap::new());
        }

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
            ControlFlow::Break(()) => return Ok(BTreeMap::new()),
            ControlFlow::Continue(old_computation_end_time) => old_computation_end_time,
        };

        // test for calmness, the fixed point must be in closed form
        // and also already once computed
        if self.process_calm(fixed_point_index, current_computation_index, start_time)? {
            return Ok(BTreeMap::new());
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

        let ground_value = CheckValue::eigen(ParamValuation::from_bool(op.is_greatest));
        let history = select_history_mut(&mut self.property_checker.histories, fixed_point_index);
        trace!("Focus: {:?}", self.property_checker.focus);
        for state_id in self.property_checker.focus.dirty_iter() {
            // clear later times
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
        let history = select_history(&self.property_checker.histories, fixed_point_index);
        let mut result = BTreeMap::new();
        for state_id in self.property_checker.focus.dirty_iter() {
            let state_history = history
                .for_state(state_id)
                .expect("Dirty state should have its history");

            let previous_update = state_history.range(0..start_time).next_back();
            let current_update = state_history.range(0..self.current_time).next_back();

            if previous_update != current_update {
                // the update does not preserve the timing value, reconstruct it
                // it is faster to do the reconstruction once per fixed-point computation
                // rather than handle it throughout every iteration
                let timed = self
                    .getter()
                    .compute_latest_timed(fixed_point_index, state_id)?;

                result.insert(state_id, timed);
            }
        }

        Ok(result)
    }
}

fn select_history_mut(
    histories: &mut BTreeMap<usize, FixedPointHistory>,
    fixed_point_index: usize,
) -> &mut FixedPointHistory {
    histories
        .get_mut(&fixed_point_index)
        .expect("Fixed point histories should contain property")
}

fn select_history(
    histories: &BTreeMap<usize, FixedPointHistory>,
    fixed_point_index: usize,
) -> &FixedPointHistory {
    histories
        .get(&fixed_point_index)
        .expect("Fixed point histories should contain property")
}
