mod fixed_point;
mod local;
mod next;

use std::result;

use log::trace;
use machine_check_common::{iir::ISubproperty, ExecError, StateId};

pub use local::BiChoice;
use mck::three_valued::ThreeValued;

use crate::{
    model_check::property_checker::{
        value::{CheckValue, TimedCheckValue},
        PropertyChecker,
    },
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
        trace!(
            "Computing subproperty {} for state {}",
            subproperty_index,
            state_id
        );

        let subproperty_entry = self
            .property_checker
            .property
            .subproperty_entry(subproperty_index);

        /*let ty = subproperty_entry.ty.clone();

        let result = match &ty {
            PropertyType::Const(constant) => {
                let value = CheckValue::from_bool(*constant);
                TimedCheckValue::new(0, value)
            }

            PropertyType::Atomic(atomic_property) => {
                let three_valued = self.space.atomic_label(atomic_property, state_id)?;
                let value = match three_valued {
                    ThreeValued::False => CheckValue::False,
                    ThreeValued::True => CheckValue::True,
                    ThreeValued::Unknown => CheckValue::Unknown(vec![]),
                };

                TimedCheckValue::new(0, value)
            }

            PropertyType::Negation(inner) => self.compute_negation(*inner, state_id)?,
            PropertyType::BiLogic(op) => self.compute_binary_op(op, state_id)?,
            PropertyType::Next(op) => self.compute_next_labelling(op, state_id.into())?,
            PropertyType::FixedPoint(op) => self.compute_fixed_point_op(op, state_id)?,
            PropertyType::FixedVariable(fixed_point_index) => {
                self.compute_fixed_variable(*fixed_point_index, state_id)?
            }
        };*/

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
