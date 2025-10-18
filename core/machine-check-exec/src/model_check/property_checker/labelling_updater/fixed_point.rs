use std::{collections::BTreeMap, ops::ControlFlow};

use log::{debug, trace};
use machine_check_common::{iir::ISubpropertyFixedPoint, ExecError, StateId};

use crate::{
    model_check::property_checker::{
        history::FixedPointHistory, labelling_updater::LabellingUpdater, CheckChoice, CheckValue,
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
}

impl<M: FullMachine> LabellingUpdater<'_, M> {
    pub fn update_fixed_point_op(
        &mut self,
        fixed_point_index: usize,
        op: &ISubpropertyFixedPoint,
    ) -> Result<BTreeMap<StateId, CheckValue>, ExecError> {
        trace!("Updating fixed-point labelling");

        if self.invalidate {
            trace!("Fixed-point immediately invalidated");
            // just invalidate fast
            return Ok(BTreeMap::new());
        }

        debug!(
            "Computing fixed point {} with {}/{} states dirty",
            fixed_point_index,
            self.property_checker.focus.dirty().len(),
            self.space.num_states(),
        );

        // update the dirty states to ground values
        // note that if there was no old computation, all states in the state space have been made dirty

        let ground_value = CheckValue::fixed_from_bool(op.universal);
        let history = select_history_mut(&mut self.property_checker.histories, fixed_point_index);
        trace!("Focus: {:?}", self.property_checker.focus);

        for state_id in self.property_checker.focus.dirty_iter() {
            history.insert(state_id, ground_value.clone());
        }

        // iterate until the fixed point is reached

        let mut params = FixedPointIterationParams {
            fixed_point_index,
            inner_index: op.inner,
        };

        while let ControlFlow::Continue(()) = self.fixed_point_iteration(&mut params)? {}

        // we reached the fixed point
        // the inner updated have been cleared

        debug!(
            "Reached fixed point {} with {}/{} states dirty,  history: {:?}",
            fixed_point_index,
            self.property_checker.focus.dirty().len(),
            self.space.num_states(),
            select_history_mut(&mut self.property_checker.histories, fixed_point_index)
        );

        let mut result = BTreeMap::new();

        for state_id in self.property_checker.focus.dirty_iter() {
            let value = self
                .property_checker
                .get_history(fixed_point_index)
                .require(state_id);
            result.insert(
                state_id,
                CheckValue {
                    valuation: value.valuation,
                    choice: CheckChoice::FixedVariable,
                },
            );
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
