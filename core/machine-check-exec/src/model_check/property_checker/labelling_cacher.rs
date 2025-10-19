mod fixed_point;
mod local;
mod next;

use log::trace;
use machine_check_common::{iir::ISubproperty, ExecError, StateId};

use crate::{
    model_check::property_checker::{value::TimedCheckValue, PropertyChecker},
    space::StateSpace,
    FullMachine,
};

pub struct LabellingCacher<'a, M: FullMachine> {
    property_checker: &'a PropertyChecker,
    space: &'a StateSpace<M>,
    current_time: u64,
}

impl<'a, M: FullMachine> LabellingCacher<'a, M> {
    pub(super) fn new(
        property_checker: &'a PropertyChecker,
        space: &'a StateSpace<M>,
        current_time: u64,
    ) -> Self {
        LabellingCacher {
            property_checker,
            space,
            current_time,
        }
    }

    pub fn compute_latest_timed(
        &self,
        subproperty_index: usize,
        state_id: StateId,
    ) -> Result<TimedCheckValue, ExecError> {
        trace!(
            "Computing subproperty {} for state {}",
            subproperty_index,
            state_id
        );

        let subproperty_entry = self
            .property_checker
            .property
            .subproperty_entry(subproperty_index);

        let result = match &subproperty_entry {
            ISubproperty::Func(subproperty) => self.compute_func(subproperty, state_id)?,
            ISubproperty::Next(subproperty) => {
                self.compute_next_labelling(subproperty, state_id.into())?
            }
            ISubproperty::FixedPoint(subproperty) => {
                self.compute_fixed_point_op(subproperty, state_id)?
            }
        };

        Ok(result)
    }

    pub fn property_checker(&self) -> &PropertyChecker {
        self.property_checker
    }
}
