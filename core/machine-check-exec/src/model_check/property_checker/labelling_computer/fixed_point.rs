use std::{collections::BTreeMap, ops::ControlFlow};

use log::trace;
use machine_check_common::{property::FixedPointOperator, ExecError, StateId, ThreeValued};

use crate::{
    model_check::property_checker::{
        labelling_computer::LabellingComputer, CheckValue, FixedPointComputation, TimedCheckValue,
    },
    FullMachine,
};

struct FixedPointIterationParams {
    fixed_point_index: usize,
    inner_index: usize,
}

impl<M: FullMachine> LabellingComputer<'_, M> {
    pub fn compute_fixed_point_op(
        &mut self,
        fixed_point_index: usize,
        op: &FixedPointOperator,
    ) -> Result<BTreeMap<StateId, TimedCheckValue>, ExecError> {
        if self.invalidate {
            // just invalidate fast
            return Ok(BTreeMap::new());
        }

        let start_time = self.current_time;

        let current_computation_index = self.next_computation_index;
        self.next_computation_index += 1;

        let old_computation = self
            .property_checker
            .computations
            .get(current_computation_index)
            .cloned();

        let had_old_computation = old_computation.is_some();

        if let Some(old_computation) = &old_computation {
            if old_computation.fixed_point_index != fixed_point_index
                || old_computation.start_time != start_time
            {
                // invalidate and return
                trace!("Invalidating as computation does not match");
                self.invalidate = true;
                return Ok(BTreeMap::new());
            }
            trace!(
                "Old computation present, only considering dirty states {:?}",
                self.property_checker.dirty_states
            );
        } else {
            assert_eq!(
                current_computation_index,
                self.property_checker.computations.len()
            );
            self.property_checker
                .computations
                .push(FixedPointComputation {
                    fixed_point_index,
                    start_time,
                    end_time: start_time,
                });
        }

        if self.calmable_fixed_points.contains(&fixed_point_index)
            && self.is_calm(fixed_point_index, &mut Vec::new())
        {
            let calm_computation = FixedPointComputation {
                fixed_point_index,
                start_time,
                end_time: start_time,
            };

            if let Some(old_computation) = old_computation {
                if old_computation != calm_computation {
                    // invalidate and return
                    trace!("Invalidating as calm fixed-point computation does not match");
                    self.invalidate = true;
                    return Ok(BTreeMap::new());
                }
            }

            trace!(
                "Not computing fixed point {} as it is calm",
                fixed_point_index
            );
            return Ok(BTreeMap::new());
        }

        // update history to ground values

        let ground_value = CheckValue::eigen(ThreeValued::from_bool(op.is_greatest));

        let mut params = FixedPointIterationParams {
            fixed_point_index,
            inner_index: op.inner,
        };

        let history = self
            .property_checker
            .histories
            .get_mut(&params.fixed_point_index)
            .expect("Fixed point histories should contain property");

        trace!(
            "Fixed point index {}, current computation index {}, start time {}",
            fixed_point_index,
            current_computation_index,
            start_time
        );

        trace!("Computations: {:?}", self.property_checker.computations);

        if had_old_computation {
            // do not consider all states, just the dirty ones
            for state_id in self.property_checker.dirty_states.iter().cloned() {
                let old_value = history
                    .before_time_opt(start_time, state_id)
                    .map(|timed| timed.value);

                if Some(&ground_value) == old_value.as_ref() {
                    continue;
                }
                // insert the ground state, it is already dirty
                history.insert(start_time, state_id, ground_value.clone());
            }
        } else {
            trace!("Old computation not present, considering all states");
            // consider all states
            for state_id in self.space.states() {
                history.insert(start_time, state_id, ground_value.clone());
                self.property_checker.dirty_states.insert(state_id);
            }
        }

        // compute inner property labelling and update variable labelling until the fixpoint is reached
        while let ControlFlow::Continue(()) = self.fixed_point_iteration(&mut params)? {}

        // we reached the fixed point
        // the inner updated have been cleared

        trace!("Fixed point {:?} reached", params.fixed_point_index);

        let history = self
            .property_checker
            .histories
            .get_mut(&fixed_point_index)
            .expect("Fixed point should have history");

        // TODO: do not propagate all states
        let mut result = BTreeMap::new();
        for state_id in self.property_checker.dirty_states.iter().copied() {
            result.insert(state_id, history.before_time(self.current_time, state_id));
        }

        let computation = self
            .property_checker
            .computations
            .get_mut(current_computation_index)
            .expect("Current computation should be inserted");

        if had_old_computation {
            trace!(
                "Current computation [{}, {}], old computation [{},{}]",
                start_time,
                self.current_time,
                computation.start_time,
                computation.end_time
            );

            if computation.end_time < self.current_time {
                // invalidate and return
                trace!(
                    "Invalidating as old computation end time is lesser: ours {}, old {}",
                    self.current_time,
                    computation.end_time
                );
                self.invalidate = true;
                return Ok(BTreeMap::new());
            }

            // move the current time in line with computation end time
            self.current_time = computation.end_time;
        } else {
            // TODO: add padding time
            // this makes verification unsound for some reason?
            /*const PADDING_TIME: u64 = 128;
            self.current_time += PADDING_TIME;
            computation.end_time = self.current_time;*/
        }

        self.calmable_fixed_points.insert(fixed_point_index);

        Ok(result)
    }

    fn fixed_point_iteration(
        &mut self,
        params: &mut FixedPointIterationParams,
    ) -> Result<ControlFlow<(), ()>, ExecError> {
        trace!("Fixed point {:?} not reached yet", params.fixed_point_index);
        trace!("Histories: {:?}", self.property_checker.histories);

        // increment time
        self.current_time += 1;

        let mut current_update = self.compute_labelling(params.inner_index)?;

        // absence of an update does not guarantee that the dirty value does not change
        let dirty_values = self
            .getter()
            .get_labelling(params.inner_index, &self.property_checker.dirty_states)?;
        current_update.extend(dirty_values);

        let history = self
            .property_checker
            .histories
            .get_mut(&params.fixed_point_index)
            .expect("Fixed point should have history");

        let mut fixed_point_reached = true;

        for (state_id, update_timed) in current_update {
            // check if the update differs

            let old_timed = history.before_time(self.current_time, state_id);

            if update_timed.value.valuation == old_timed.value.valuation {
                continue;
            }

            // the update differs, make sure to ensure the loop does not break and increment the time once
            fixed_point_reached = false;

            // insert the state and make it dirty if the history changed
            if history.insert(self.current_time, state_id, update_timed.value) {
                self.property_checker.dirty_states.insert(state_id);
            }
        }

        if fixed_point_reached {
            return Ok(ControlFlow::Break(()));
        }

        Ok(ControlFlow::Continue(()))
    }

    pub fn compute_fixed_variable(
        &mut self,
        fixed_point_index: usize,
    ) -> Result<BTreeMap<StateId, TimedCheckValue>, ExecError> {
        // TODO: do not update all states
        self.getter()
            .get_fixed_variable(fixed_point_index, &self.property_checker.dirty_states)
    }
}
