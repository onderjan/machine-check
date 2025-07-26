use std::{
    collections::{BTreeMap, BTreeSet},
    ops::ControlFlow,
};

use log::trace;
use machine_check_common::{property::FixedPointOperator, ExecError, StateId, ThreeValued};

use crate::{
    model_check::property_checker::{
        labelling_computer::LabellingComputer, CheckValue, TimedCheckValue,
    },
    FullMachine,
};

struct FixedPointIterationParams<'a> {
    fixed_point_index: usize,
    op: &'a FixedPointOperator,
}

impl<M: FullMachine> LabellingComputer<'_, M> {
    pub fn compute_fixed_point_op(
        &mut self,
        fixed_point_index: usize,
        op: &FixedPointOperator,
    ) -> Result<BTreeMap<StateId, TimedCheckValue>, ExecError> {
        /*if self.is_calm(subproperty_index, &mut Vec::new()) {
            trace!(
                "Not computing fixed point {} as it is calm",
                subproperty_index
            );
            return Ok(BTreeSet::new());
        }*/

        // update history to ground values

        let ground_value = CheckValue::eigen(ThreeValued::from_bool(op.is_greatest));

        let mut params = FixedPointIterationParams {
            fixed_point_index,
            op,
        };

        let history = self
            .property_checker
            .fixed_point_histories
            .get_mut(&params.fixed_point_index)
            .expect("Fixed point histories should contain property");

        // TODO: do not consider all states
        for state_id in self.space.states() {
            let old_value = history
                .up_to_time_opt(self.current_time, state_id)
                .map(|timed| timed.value);

            if Some(&ground_value) == old_value.as_ref() {
                continue;
            }
            // insert the ground state, make it dirty if it was inserted
            if history.insert(self.current_time, state_id, ground_value.clone()) {
                self.property_checker.dirty_states.insert(state_id);
            }
        }

        // compute inner property labelling and update variable labelling until the fixpoint is reached
        while let ControlFlow::Continue(()) = self.fixed_point_iteration(&mut params)? {}

        self.fixed_point_conclusion(params)?;

        let history = self
            .property_checker
            .fixed_point_histories
            .get(&fixed_point_index)
            .expect("Fixed point should have history");

        // TODO: do not propagate all states
        let mut result = BTreeMap::new();
        for state_id in self.space.states() {
            result.insert(state_id, history.up_to_time(self.current_time, state_id));
        }

        Ok(result)
    }

    fn fixed_point_iteration(
        &mut self,
        params: &mut FixedPointIterationParams,
    ) -> Result<ControlFlow<(), ()>, ExecError> {
        trace!("Fixed point {:?} not reached yet", params.fixed_point_index);

        let current_update = self.compute_labelling(params.op.inner)?;

        let history = self
            .property_checker
            .fixed_point_histories
            .get_mut(&params.fixed_point_index)
            .expect("Fixed point should have history");

        let mut fixed_point_reached = true;
        //let mut control_flow = ControlFlow::Break(());

        let next_time = self.current_time + 1;

        for (state_id, update_timed) in current_update {
            // check if the update differs

            let old_timed = history.up_to_time(self.current_time, state_id);

            if update_timed.value.valuation == old_timed.value.valuation {
                continue;
            }

            // the update differs, make sure to ensure the loop does not break and increment the time once
            fixed_point_reached = false;
            /*if matches!(control_flow, ControlFlow::Break(_)) {
                control_flow = ControlFlow::Continue(());
            }*/

            // insert the state and make it dirty if the history changed
            if history.insert(next_time, state_id, update_timed.value) {
                self.property_checker.dirty_states.insert(state_id);
            }
        }

        // TODO: make the fixed-point starts and ends consistent with history

        if fixed_point_reached {
            return Ok(ControlFlow::Break(()));
        }

        // move to next time and continue the loop
        self.current_time = next_time;
        Ok(ControlFlow::Continue(()))
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
        fixed_point_index: usize,
    ) -> Result<BTreeMap<StateId, TimedCheckValue>, ExecError> {
        // TODO: do not update all states
        self.getter()
            .get_fixed_variable(fixed_point_index, &BTreeSet::from_iter(self.space.states()))
    }
}
