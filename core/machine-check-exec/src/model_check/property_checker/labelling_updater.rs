mod fixed_point;
mod local;
mod next;

use std::collections::BTreeSet;

use log::trace;
use machine_check_common::{property::PropertyType, ExecError, ThreeValued};
use mck::concr::FullMachine;

use crate::{
    model_check::property_checker::{labelling_cacher::LabellingCacher, PropertyChecker},
    space::StateSpace,
};

pub struct LabellingUpdater<'a, M: FullMachine> {
    property_checker: &'a mut PropertyChecker,
    space: &'a StateSpace<M>,

    current_time: u64,
    next_computation_index: usize,

    invalidate: bool,

    calmable_fixed_points: BTreeSet<usize>,
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
        };

        Ok(computer)
    }

    fn getter(&self) -> LabellingCacher<M> {
        LabellingCacher::new(self.property_checker, self.space, self.current_time)
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

        let mut result = ThreeValued::True;

        for state_id in self.space.initial_iter() {
            self.getter().cache_if_uncached(0, state_id)?;

            let timed = self.property_checker.get_cached(0, state_id);

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
        self.property_checker.latest_cache.get_mut().clear_all();
        self.update_labelling(0)?;
        Ok(())
    }

    fn update_labelling(&mut self, subproperty_index: usize) -> Result<(), ExecError> {
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

        match &ty {
            PropertyType::Const(_) | PropertyType::Atomic(_) => {
                if self.current_time == 0 {
                    // only update for dirty states
                    for state_id in self.property_checker.focus.dirty() {
                        self.getter().force_recache(subproperty_index, *state_id)?;
                    }
                }
            }
            PropertyType::Negation(inner) => self.update_negation(*inner)?,
            PropertyType::BiLogic(op) => self.update_binary_op(op)?,
            PropertyType::Next(op) => self.update_next_labelling(op)?,
            PropertyType::FixedPoint(op) => self.update_fixed_point_op(subproperty_index, op)?,
            PropertyType::FixedVariable(fixed_point_index) => {
                self.update_fixed_variable(*fixed_point_index)?
            }
        }

        trace!("Subproperty {:?} labelling updated", subproperty_index);

        Ok(())
    }
}
