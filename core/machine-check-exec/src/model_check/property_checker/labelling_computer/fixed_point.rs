use std::{collections::BTreeMap, ops::ControlFlow};

use log::trace;
use machine_check_common::{property::FixedPointOperator, ExecError, StateId, ThreeValued};

use crate::{
    model_check::property_checker::{
        history::FixedPointHistory, labelling_computer::LabellingComputer, CheckValue,
        FixedPointComputation, TimedCheckValue,
    },
    FullMachine,
};

struct FixedPointIterationParams {
    fixed_point_index: usize,
    inner_index: usize,
    required_end_time: Option<u64>,
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

        // either check old computation (excluding end time) or add new computation

        let current_computation_index = self.next_computation_index;
        self.next_computation_index += 1;

        let old_computation_end_time =
            match current_computation_index.cmp(&self.property_checker.computations.len()) {
                std::cmp::Ordering::Less => {
                    // we have an old computation
                    let old_computation =
                        &self.property_checker.computations[current_computation_index];
                    if old_computation.fixed_point_index != fixed_point_index
                        || old_computation.start_time != start_time
                    {
                        // invalidate and return
                        trace!("Invalidating as computation does not match");
                        self.invalidate = true;
                        return Ok(BTreeMap::new());
                    }
                    trace!(
                        "Old computation present, only considering focus {:?}",
                        self.property_checker.focus
                    );
                    Some(old_computation.end_time)
                }
                std::cmp::Ordering::Equal => {
                    // we do not have an old computation
                    // all states must be dirty
                    self.property_checker
                        .focus
                        .extend_dirty(self.space, self.space.states());

                    // add the computation
                    self.property_checker
                        .computations
                        .push(FixedPointComputation {
                            fixed_point_index,
                            start_time,
                            end_time: start_time,
                        });
                    assert_eq!(
                        self.next_computation_index,
                        self.property_checker.computations.len()
                    );
                    None
                }
                std::cmp::Ordering::Greater => {
                    panic!(
                        "Computation index {} > {} is not permitted",
                        current_computation_index,
                        self.property_checker.computations.len()
                    );
                }
            };

        // test for calmness, the fixed point must be in closed form
        // and also already once computed

        if self
            .property_checker
            .closed_form_subproperties
            .contains(&fixed_point_index)
            && self.calmable_fixed_points.contains(&fixed_point_index)
        {
            // this fixed point is calm, it should have the same start and end time
            let current_computation =
                &self.property_checker.computations[current_computation_index];
            if current_computation.end_time != start_time {
                // invalidate and return
                trace!("Invalidating as calm fixed-point computation does not match");
                self.invalidate = true;
                return Ok(BTreeMap::new());
            }

            trace!(
                "Not computing fixed point {} as it is calm",
                fixed_point_index
            );
            return Ok(BTreeMap::new());
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
            required_end_time: old_computation_end_time,
        };

        while let ControlFlow::Continue(()) = self.fixed_point_iteration(&mut params)? {}

        // we reached the fixed point
        // the inner updated have been cleared

        trace!("Fixed point {:?} reached", params.fixed_point_index);

        let computation_clone =
            self.property_checker.computations[current_computation_index].clone();

        if old_computation_end_time.is_some() {
            trace!(
                "Current computation [{}, {}], old computation [{},{}]",
                start_time,
                self.current_time,
                computation_clone.start_time,
                computation_clone.end_time
            );

            assert!(computation_clone.start_time < computation_clone.end_time);

            // replay the remaining fixed-point computations inside
            /*while let Some(replay_computation) = self
                .property_checker
                .computations
                .get(self.next_computation_index)
            {
                if replay_computation.end_time > computation_clone.end_time {
                    // stop replaying
                    break;
                }

                trace!(
                    "Replaying fixed point index {} at time {:?}",
                    replay_computation.fixed_point_index, self.current_time
                );

                self.compute_fixed_point_op(replay_computation.fixed_point_index, op)?;

                if self.invalidate {
                    // the index does not move when invalidated, break out
                    return Ok(BTreeMap::new());
                }
            }*/

            // after replaying, we can move the current time

            if self.current_time != computation_clone.end_time {
                // invalidate and return
                trace!(
                    "Invalidating as old computation end time is lesser: ours {}, old {}",
                    self.current_time,
                    computation_clone.end_time
                );
                self.invalidate = true;
                return Ok(BTreeMap::new());
            }
            /*if self.current_time < computation_clone.end_time {
                trace!("Moving computation end time of fixed point {} with start time {} from {} to {}", fixed_point_index, start_time, self.current_time, computation_clone.end_time);
            }

            // move the current time in line with computation end time
            self.current_time = computation_clone.end_time;*/
        } else {
            // TODO: add padding time
            // this makes verification unsound for some reason?
            /*const PADDING_TIME: u64 = 128;
            self.current_time += PADDING_TIME;*/

            assert_eq!(computation_clone.start_time, computation_clone.end_time);
            assert!(computation_clone.end_time < self.current_time);

            self.property_checker.computations[current_computation_index].end_time =
                self.current_time;
        }

        // since this fixed point was computed properly, it can be considered when calming afterwards
        self.calmable_fixed_points.insert(fixed_point_index);

        // select the states to propagate
        // TODO: do not propagate all affected states
        let history = select_history(&mut self.property_checker.histories, fixed_point_index);
        let mut result = BTreeMap::new();
        for state_id in self.property_checker.focus.affected().iter().copied() {
            result.insert(state_id, history.before_time(self.current_time, state_id));
        }
        trace!(
            "End computation index {}, next computation index {}, length {}",
            current_computation_index,
            self.next_computation_index,
            self.property_checker.computations.len()
        );

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

        trace!("Current update: {:?}", current_update);

        // absence of an update does not guarantee that the dirty value does not change
        let dirty_values = self
            .getter()
            .get_labelling(params.inner_index, self.property_checker.focus.dirty())?;

        trace!("Dirty values: {:?}", dirty_values);
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
                self.property_checker
                    .focus
                    .insert_dirty(self.space, state_id);
            }
        }

        if let Some(required_end_time) = params.required_end_time {
            if self.current_time < required_end_time {
                trace!(
                    "Current time {}, required end time {}, waiting",
                    self.current_time,
                    required_end_time
                );
                return Ok(ControlFlow::Continue(()));
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
        // return the values of affected, not just dirty
        self.getter()
            .get_fixed_variable(fixed_point_index, self.property_checker.focus.affected())
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
