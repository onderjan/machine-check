use std::{
    collections::{BTreeMap, BTreeSet},
    ops::ControlFlow,
};

use log::trace;
use machine_check_common::{property::FixedPointOperator, ExecError, StateId, ThreeValued};

use crate::{
    model_check::property_checker::{
        history::TimeSpan, labelling_computer::LabellingComputer, CheckValue, TimedCheckValue,
    },
    FullMachine,
};

struct FixedPointIterationParams<'a> {
    start_time: u64,
    fixed_point_index: usize,
    op: &'a FixedPointOperator,
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
        /*if self.is_calm(subproperty_index, &mut Vec::new()) {
            trace!(
                "Not computing fixed point {} as it is calm",
                subproperty_index
            );
            return Ok(BTreeSet::new());
        }*/

        // update history to ground values

        let ground_value = CheckValue::eigen(ThreeValued::from_bool(op.is_greatest));

        let start_time = self.current_time;

        let mut params = FixedPointIterationParams {
            fixed_point_index,
            op,
            start_time,
        };

        let history = self
            .property_checker
            .fixed_point_histories
            .get_mut(&params.fixed_point_index)
            .expect("Fixed point histories should contain property");

        let current_computation_index = self
            .fixed_point_next_computations
            .entry(params.fixed_point_index)
            .or_default();

        trace!(
            "Fixed point index {}, current computation index {}, start time {}",
            fixed_point_index,
            current_computation_index,
            start_time
        );

        trace!("History computations: {:?}", history.computations);

        if let Some(old_computation) = history.computations.get(*current_computation_index) {
            if old_computation.start_time != start_time {
                trace!("Invalidating as computation start time does not match");
                // invalidate and return
                self.invalidate = true;
                return Ok(BTreeMap::new());
            }
            trace!(
                "Old computation present, only considering dirty states {:?}",
                self.property_checker.dirty_states
            );

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
            .fixed_point_histories
            .get_mut(&fixed_point_index)
            .expect("Fixed point should have history");

        let end_time = self.current_time;

        let current_computation = self
            .fixed_point_next_computations
            .entry(params.fixed_point_index)
            .or_default();

        if let Some(old_computation) = history.computations.get(*current_computation) {
            if old_computation.end_time != end_time {
                // invalidate and return
                trace!("Invalidating as computation end time does not match");
                self.invalidate = true;
                return Ok(BTreeMap::new());
            }
            trace!(
                "Current computation [{}, {}], old computation [{},{}]",
                start_time,
                end_time,
                old_computation.start_time,
                old_computation.end_time
            );
        } else {
            history.computations.push(TimeSpan {
                start_time: params.start_time,
                end_time,
            });
        }

        *current_computation += 1;

        // TODO: do not propagate all states
        let mut result = BTreeMap::new();
        for state_id in self.space.states() {
            result.insert(state_id, history.before_time(self.current_time, state_id));
        }

        Ok(result)
    }

    fn fixed_point_iteration(
        &mut self,
        params: &mut FixedPointIterationParams,
    ) -> Result<ControlFlow<(), ()>, ExecError> {
        trace!("Fixed point {:?} not reached yet", params.fixed_point_index);

        // increment time
        self.current_time += 1;

        let mut current_update = self.compute_labelling(params.op.inner)?;

        // absence of an update does not guarantee that the dirty value does not change
        let dirty_values = self
            .getter()
            .get_labelling(params.op.inner, &self.property_checker.dirty_states)?;
        current_update.extend(dirty_values);

        let history = self
            .property_checker
            .fixed_point_histories
            .get_mut(&params.fixed_point_index)
            .expect("Fixed point should have history");

        let mut fixed_point_reached = true;
        //let mut control_flow = ControlFlow::Break(());

        for (state_id, update_timed) in current_update {
            // check if the update differs

            let old_timed = history.before_time(self.current_time, state_id);

            if update_timed.value.valuation == old_timed.value.valuation {
                continue;
            }

            // the update differs, make sure to ensure the loop does not break and increment the time once
            fixed_point_reached = false;
            /*if matches!(control_flow, ControlFlow::Break(_)) {
                control_flow = ControlFlow::Continue(());
            }*/

            // insert the state and make it dirty if the history changed
            if history.insert(self.current_time, state_id, update_timed.value) {
                self.property_checker.dirty_states.insert(state_id);
            }
        }

        // TODO: make the fixed-point starts and ends consistent with history

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
            .get_fixed_variable(fixed_point_index, &BTreeSet::from_iter(self.space.states()))
    }
}
