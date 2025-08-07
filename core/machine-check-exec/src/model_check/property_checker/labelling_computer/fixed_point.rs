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

        let had_old_computation =
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
                        "Old computation present, only considering dirty states {:?}",
                        self.property_checker.dirty_states
                    );
                    true
                }
                std::cmp::Ordering::Equal => {
                    // we do not have an old computation
                    // all states must be dirty
                    self.property_checker
                        .dirty_states
                        .extend(self.space.states());

                    // add the computation
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
                    false
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
        for state_id in self.property_checker.dirty_states.iter().cloned() {
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

        let computation = &mut self.property_checker.computations[current_computation_index];

        if had_old_computation {
            trace!(
                "Current computation [{}, {}], old computation [{},{}]",
                start_time,
                self.current_time,
                computation.start_time,
                computation.end_time
            );

            if computation.end_time != self.current_time {
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
            self.current_time += PADDING_TIME;*/

            computation.end_time = self.current_time;
        }

        // since this fixed point was computed properly, it can be considered when calming afterwards
        self.calmable_fixed_points.insert(fixed_point_index);

        // select the states to propagate
        // TODO: do not propagate all dirty states
        let history = select_history(&mut self.property_checker.histories, fixed_point_index);
        let mut result = BTreeMap::new();
        for state_id in self.property_checker.dirty_states.iter().copied() {
            result.insert(state_id, history.before_time(self.current_time, state_id));
        }

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

fn select_history(
    histories: &mut BTreeMap<usize, FixedPointHistory>,
    fixed_point_index: usize,
) -> &mut FixedPointHistory {
    histories
        .get_mut(&fixed_point_index)
        .expect("Fixed point histories should contain property")
}
