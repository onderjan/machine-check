mod fixed_point;
mod local;
mod next;

use std::collections::{BTreeMap, BTreeSet};

use log::trace;
use machine_check_common::{property::PropertyType, ExecError, StateId, ThreeValued};
use mck::concr::FullMachine;

use crate::{
    model_check::property_checker::{
        labelling_getter::LabellingGetter, PropertyChecker, TimedCheckValue,
    },
    space::StateSpace,
};

pub struct LabellingComputer<'a, M: FullMachine> {
    property_checker: &'a mut PropertyChecker,
    space: &'a StateSpace<M>,

    current_time: u64,
}

impl<'a, M: FullMachine> LabellingComputer<'a, M> {
    pub fn new(
        property_checker: &'a mut PropertyChecker,
        space: &'a StateSpace<M>,
    ) -> Result<Self, ExecError> {
        property_checker.reinit_labellings()?;

        let computer = Self {
            property_checker,
            space,
            current_time: 0,
        };

        Ok(computer)
    }

    fn getter(&self) -> LabellingGetter<M> {
        LabellingGetter::new(self.property_checker, self.space, self.current_time)
    }

    pub fn compute(&mut self) -> Result<ThreeValued, ExecError> {
        self.current_time = 0;

        self.compute_labelling(0)?;

        self.property_checker.recompute_states.clear();

        // conventionally, the property must hold in all initial states

        let values = self
            .getter()
            .get_labelling(0, &BTreeSet::from_iter(self.space.initial_iter()))?;

        let mut result = ThreeValued::True;
        for (_initial_state_id, timed) in values {
            let valuation = timed.value.valuation;
            result = result & valuation;
        }

        trace!(
            "Computed interpretation of {:?}",
            self.property_checker.property
        );

        Ok(result)
    }

    fn compute_labelling(
        &mut self,
        subproperty_index: usize,
    ) -> Result<BTreeMap<StateId, TimedCheckValue>, ExecError> {
        let subproperty_entry = self
            .property_checker
            .property
            .subproperty_entry(subproperty_index);

        let ty = subproperty_entry.ty.clone();

        let updated = match &ty {
            PropertyType::Const(_) | PropertyType::Atomic(_) => {
                // TODO: do not update all states
                self.getter()
                    .get_labelling(subproperty_index, &BTreeSet::from_iter(self.space.states()))?
            }
            PropertyType::Negation(inner) => self.compute_negation(*inner)?,
            PropertyType::BiLogic(op) => self.compute_binary_op(op)?,
            PropertyType::Next(op) => self.compute_next_labelling(op)?,
            PropertyType::FixedPoint(op) => self.compute_fixed_point_op(subproperty_index, op)?,
            PropertyType::FixedVariable(fixed_point_index) => {
                self.compute_fixed_variable(*fixed_point_index)?
            }
        };

        trace!(
            "Subproperty {:?} labelling computed, updated: {:?}",
            subproperty_index,
            updated
        );

        Ok(updated)
    }

    /*
    pub fn is_calm(&self, subproperty_index: usize, calm_fixed_points: &mut Vec<usize>) -> bool {
        let update = Self::computation(&self.updates, subproperty_index);
        if !update.is_empty() {
            trace!(
                "Subproperty {} is not calm as it has updates",
                subproperty_index
            );
            return false;
        }

        let subproperty_entry = self
            .property_checker
            .property
            .subproperty_entry(subproperty_index);

        let result = match &subproperty_entry.ty {
            PropertyType::Const(_) | PropertyType::Atomic(_) => true,
            PropertyType::Negation(inner) => self.is_calm(*inner, calm_fixed_points),
            PropertyType::BiLogic(bi_logic_operator) => {
                self.is_calm(bi_logic_operator.a, calm_fixed_points)
                    && self.is_calm(bi_logic_operator.b, calm_fixed_points)
            }
            PropertyType::Next(next_operator) => {
                self.is_calm(next_operator.inner, calm_fixed_points)
            }
            PropertyType::FixedPoint(fixed_point_operator) => {
                calm_fixed_points.push(subproperty_index);
                let result = self.is_calm(fixed_point_operator.inner, calm_fixed_points);
                calm_fixed_points.pop();
                result
            }
            PropertyType::FixedVariable(fixed_point_index) => {
                calm_fixed_points.contains(fixed_point_index)
            }
        };

        trace!("Subproperty {} calmness: {}", subproperty_index, result);

        result
    }*/
}
