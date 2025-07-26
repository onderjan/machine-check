mod fixed_point;
mod local;
mod next;

use std::collections::{BTreeMap, BTreeSet};

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

    pub fn get_labelling(
        &self,
        subproperty_index: usize,
        states: &BTreeSet<StateId>,
    ) -> Result<BTreeMap<StateId, TimedCheckValue>, ExecError> {
        let subproperty_entry = self
            .property_checker
            .property
            .subproperty_entry(subproperty_index);

        let ty = subproperty_entry.ty.clone();

        let updated = match &ty {
            PropertyType::Const(constant) => {
                let constant = ThreeValued::from_bool(*constant);
                let eigen = CheckValue::eigen(constant);
                let timed = TimedCheckValue {
                    time: 0,
                    value: eigen,
                };

                BTreeMap::from_iter(
                    states
                        .iter()
                        .copied()
                        .map(|state_id| (state_id, timed.clone())),
                )
            }
            PropertyType::Atomic(atomic_property) => {
                let mut result = BTreeMap::new();

                for state_id in states.iter().copied() {
                    let value = self.space.atomic_label(atomic_property, state_id)?;
                    let value = CheckValue::eigen(value);
                    result.insert(state_id, TimedCheckValue { time: 0, value });
                }

                result
            }
            PropertyType::Negation(inner) => self.get_negation(*inner, states)?,
            PropertyType::BiLogic(op) => self.get_binary_op(op, states)?,
            PropertyType::Next(op) => self.get_next_labelling(op, states)?,
            PropertyType::FixedPoint(op) => self.get_fixed_point_op(op, states)?,
            PropertyType::FixedVariable(fixed_point_index) => {
                self.get_fixed_variable(*fixed_point_index, states)?
            }
        };

        Ok(updated)
    }

    pub fn get_state_label(
        &self,
        subproperty_index: usize,
        state_id: StateId,
    ) -> Result<TimedCheckValue, ExecError> {
        Ok(self
            .get_labelling(subproperty_index, &BTreeSet::from([state_id]))?
            .get(&state_id)
            .expect("Single state should be in labelling")
            .clone())
    }
}
