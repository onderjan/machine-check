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
    next_computation_index: usize,
    invalidate: bool,

    calmable_fixed_points: BTreeSet<usize>,
}

impl<'a, M: FullMachine> LabellingComputer<'a, M> {
    pub fn new(
        property_checker: &'a mut PropertyChecker,
        space: &'a StateSpace<M>,
    ) -> Result<Self, ExecError> {
        let computer = Self {
            property_checker,
            space,
            current_time: 0,
            next_computation_index: 0,
            invalidate: false,
            calmable_fixed_points: BTreeSet::new(),
        };

        Ok(computer)
    }

    fn getter(&self) -> LabellingGetter<M> {
        LabellingGetter::new(self.property_checker, self.space, self.current_time)
    }

    pub fn compute(mut self) -> Result<ThreeValued, ExecError> {
        trace!("Computing, dirty states: {:?}", self.property_checker.focus);
        self.compute_inner()?;

        if self.invalidate {
            trace!("Invalidated");
            self.property_checker.invalidate();
            self.property_checker
                .focus
                .extend_dirty(self.space, self.space.states());
            self.invalidate = false;

            trace!("Invalidated computation, computing once more");

            self.compute_inner()?;
            assert!(!self.invalidate);
        } else {
            trace!("Computation not invalidated");
            self.property_checker.incremental_double_check(self.space)?;
        }

        trace!("Computed, focus: {:?}", self.property_checker.focus);

        self.property_checker.focus.clear();

        // conventionally, the property must hold in all initial states

        let values = self.getter().get_labelling(0, self.space.initial_iter())?;

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

    pub(super) fn compute_inner(&mut self) -> Result<(), ExecError> {
        self.current_time = 0;
        self.next_computation_index = 0;
        self.calmable_fixed_points.clear();
        self.compute_labelling(0)?;
        Ok(())
    }

    fn compute_labelling(
        &mut self,
        subproperty_index: usize,
    ) -> Result<BTreeMap<StateId, TimedCheckValue>, ExecError> {
        let subproperty_entry = self
            .property_checker
            .property
            .subproperty_entry(subproperty_index);

        trace!(
            "Subproperty {:?} entry: {:?}",
            subproperty_index,
            subproperty_entry
        );

        let ty = subproperty_entry.ty.clone();

        let updated = match &ty {
            PropertyType::Const(_) | PropertyType::Atomic(_) => {
                if self.current_time == 0 {
                    // only newly compute for dirty states
                    self.getter().get_labelling(
                        subproperty_index,
                        self.property_checker.focus.dirty().iter().copied(),
                    )?
                } else {
                    // this has already been computed
                    BTreeMap::new()
                }
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
}
