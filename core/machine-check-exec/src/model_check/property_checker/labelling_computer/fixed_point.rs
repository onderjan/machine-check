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

        for state_id in self.space.states() {
            self.property_checker
                .fixed_point_histories
                .get_mut(&params.fixed_point_index)
                .expect("Fixed point histories should contain property")
                .insert_update(self.current_time, state_id, ground_value.clone());
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

        let old_time = self.current_time;

        let history = self
            .property_checker
            .fixed_point_histories
            .get_mut(&params.fixed_point_index)
            .expect("Fixed point should have history");

        let mut control_flow = ControlFlow::Break(());

        for (state_id, update_timed) in current_update {
            // check if the update differs

            let old_timed = history.up_to_time(old_time, state_id);

            if update_timed.value.valuation == old_timed.value.valuation {
                continue;
            }

            // the update differs, set to continue iterating and insert
            if matches!(control_flow, ControlFlow::Break(_)) {
                control_flow = ControlFlow::Continue(());
                // update time
                self.current_time += 1;
            }

            history.insert_update(self.current_time, state_id, update_timed.value);
        }

        Ok(control_flow)
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
