mod fixed_point;
mod local;
mod next;

use machine_check_common::{property::PropertyType, ExecError, StateId, ThreeValued};

pub use local::BiChoice;

use crate::{
    model_check::property_checker::{CheckValue, PropertyChecker, TimedCheckValue},
    space::StateSpace,
    FullMachine,
};

pub struct LabellingGetter<'a, M: FullMachine> {
    property_checker: &'a PropertyChecker,
    space: &'a StateSpace<M>,
    current_time: u64,
}

impl<'a, M: FullMachine> LabellingGetter<'a, M> {
    pub(super) fn new(
        property_checker: &'a PropertyChecker,
        space: &'a StateSpace<M>,
        current_time: u64,
    ) -> Self {
        LabellingGetter {
            property_checker,
            space,
            current_time,
        }
    }

    pub fn space(&self) -> &StateSpace<M> {
        self.space
    }

    pub fn cache_labelling(
        &self,
        subproperty_index: usize,
        state_id: StateId,
    ) -> Result<(), ExecError> {
        if self
            .property_checker
            .get_cached_opt(subproperty_index, state_id)
            .is_some()
        {
            return Ok(());
        }

        self.cache_labelling_inner(subproperty_index, state_id)
    }

    pub fn update_labelling(
        &self,
        subproperty_index: usize,
        state_id: StateId,
    ) -> Result<(), ExecError> {
        self.cache_labelling_inner(subproperty_index, state_id)
    }

    pub fn cache_labelling_inner(
        &self,
        subproperty_index: usize,
        state_id: StateId,
    ) -> Result<(), ExecError> {
        let subproperty_entry = self
            .property_checker
            .property
            .subproperty_entry(subproperty_index);

        let ty = subproperty_entry.ty.clone();

        let result = match &ty {
            PropertyType::Const(constant) => {
                let constant = ThreeValued::from_bool(*constant);
                let eigen = CheckValue::eigen(constant);
                TimedCheckValue {
                    time: 0,
                    value: eigen,
                }
            }

            PropertyType::Atomic(atomic_property) => {
                let value = self.space.atomic_label(atomic_property, state_id)?;
                let value = CheckValue::eigen(value);
                TimedCheckValue { time: 0, value }
            }

            PropertyType::Negation(inner) => self.evaluate_negation(*inner, state_id)?,
            PropertyType::BiLogic(op) => self.evaluate_binary_op(op, state_id)?,
            PropertyType::Next(op) => self.cache_next_labelling(op, state_id)?,
            PropertyType::FixedPoint(op) => self.cache_fixed_point_op(op, state_id)?,
            PropertyType::FixedVariable(fixed_point_index) => {
                self.cache_fixed_variable(*fixed_point_index, state_id)?
            }
        };

        self.property_checker
            .insert_into_cache(subproperty_index, state_id, result);

        Ok(())
    }

    pub fn cache_and_get(
        &self,
        subproperty_index: usize,
        state_id: StateId,
    ) -> Result<TimedCheckValue, ExecError> {
        self.cache_labelling(subproperty_index, state_id)?;
        Ok(self
            .property_checker
            .get_cached(subproperty_index, state_id))
    }
}
