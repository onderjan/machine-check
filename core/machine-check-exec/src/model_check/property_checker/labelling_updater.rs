mod fixed_point;
mod local;
mod next;

use std::collections::{BTreeMap, BTreeSet};

use log::{debug, trace};
use machine_check_common::{property::PropertyType, ExecError, StateId};
use mck::{concr::FullMachine, three_valued::ThreeValued};

use crate::{
    model_check::property_checker::{
        history::TimedCheckValue, labelling_cacher::LabellingCacher, PropertyChecker,
    },
    space::StateSpace,
};

pub struct LabellingUpdater<'a, M: FullMachine> {
    property_checker: &'a mut PropertyChecker,
    space: &'a StateSpace<M>,

    current_time: u64,
    next_computation_index: usize,

    invalidate: bool,

    calmable_fixed_points: BTreeSet<usize>,

    num_fixed_point_computations: u64,
    num_fixed_point_iterations: u64,
}

impl<'a, M: FullMachine> LabellingUpdater<'a, M> {
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
            num_fixed_point_computations: 0,
            num_fixed_point_iterations: 0,
        };

        Ok(computer)
    }

    fn getter(&self) -> LabellingCacher<M> {
        LabellingCacher::new(self.property_checker, self.space, self.current_time)
    }

    pub fn compute(mut self) -> Result<ThreeValued, ExecError> {
        trace!(
            "Computing, focus: {:?}, state space: {:#?}",
            self.property_checker.focus,
            self.space
        );
        trace!(
            "Histories when computing: {:#?}",
            self.property_checker.histories
        );
        self.compute_inner()?;

        if self.invalidate {
            debug!("Invalidated computation, computing once more");
            self.property_checker.invalidate();
            self.property_checker.focus.make_whole_dirty(self.space);
            self.invalidate = false;

            self.compute_inner()?;
            assert!(!self.invalidate);
        } else {
            self.property_checker.incremental_double_check(self.space)?;
        }

        trace!("Computed, focus: {:?}", self.property_checker.focus);

        self.property_checker.focus.clear();

        // conventionally, the property must hold in all initial states

        let mut result = ThreeValued::True;

        for state_id in self.space.initial_iter() {
            self.getter().compute_latest_timed(0, state_id)?;

            let timed = self
                .property_checker
                .last_getter(self.space)
                .compute_latest_timed(0, state_id)?;

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
        self.num_fixed_point_computations = 0;
        self.num_fixed_point_iterations = 0;
        self.update_labelling(0)?;
        debug!(
            "Model-checked, fixed points: {} computations, {} iterations, {}/{} states dirty",
            self.num_fixed_point_computations,
            self.num_fixed_point_iterations,
            self.property_checker.focus.dirty().len(),
            self.space.num_states()
        );
        Ok(())
    }

    fn update_labelling(
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
                let mut result = BTreeMap::new();
                if self.current_time == 0 {
                    // update dirty states
                    for state_id in self.property_checker.focus.dirty_iter() {
                        let timed = self
                            .getter()
                            .compute_latest_timed(subproperty_index, state_id)?;
                        result.insert(state_id, timed);
                    }
                }
                result
            }
            PropertyType::Negation(inner) => self.update_negation(*inner)?,
            PropertyType::BiLogic(op) => self.update_binary_op(op)?,
            PropertyType::Next(op) => self.update_next_labelling(op)?,
            PropertyType::FixedPoint(op) => self.update_fixed_point_op(subproperty_index, op)?,
            PropertyType::FixedVariable(fixed_point_index) => {
                self.update_fixed_variable(*fixed_point_index)?
            }
        };

        Ok(updated)
    }
}
