mod fixed_point;
mod local;
mod next;

use machine_check_common::{property::PropertyType, ExecError, ParamValuation, StateId};

pub use local::BiChoice;

use crate::{
    model_check::property_checker::{CheckValue, PropertyChecker, TimedCheckValue},
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

    pub fn space(&self) -> &StateSpace<M> {
        self.space
    }

    pub fn compute_latest_timed(
        &self,
        subproperty_index: usize,
        state_id: StateId,
    ) -> Result<TimedCheckValue, ExecError> {
        let subproperty_entry = self
            .property_checker
            .property
            .subproperty_entry(subproperty_index);

        let ty = subproperty_entry.ty.clone();

        let result = match &ty {
            PropertyType::Const(constant) => {
                let constant = ParamValuation::from_bool(*constant);
                let value = CheckValue::eigen(constant);
                TimedCheckValue::new(0, value)
            }

            PropertyType::Atomic(atomic_property) => {
                let three_valued = self.space.atomic_label(atomic_property, state_id)?;
                let value = CheckValue::eigen(ParamValuation::from_three_valued(three_valued));
                TimedCheckValue::new(0, value)
            }

            PropertyType::Negation(inner) => self.compute_negation(*inner, state_id)?,
            PropertyType::BiLogic(op) => self.compute_binary_op(op, state_id)?,
            PropertyType::Next(op) => self.compute_next_labelling(op, state_id)?,
            PropertyType::FixedPoint(op) => self.compute_fixed_point_op(op, state_id)?,
            PropertyType::FixedVariable(fixed_point_index) => {
                self.compute_fixed_variable(*fixed_point_index, state_id)?
            }
        };

        Ok(result)
    }
}
