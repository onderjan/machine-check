mod fixed_point;
mod local;
mod next;

use std::collections::{BTreeMap, BTreeSet};

use log::{debug, trace};
use machine_check_common::{
    iir::{ISubproperty, ISubpropertyNext},
    ExecError, NodeId, ParamValuation, StateId,
};
use mck::concr::FullMachine;

use crate::{
    model_check::property_checker::{
        labelling_cacher::LabellingCacher, value::TimedCheckValue, PropertyChecker,
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

    pub fn compute(mut self) -> Result<ParamValuation, ExecError> {
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

        // for a conventional result, we compute AX of the root node
        // this will also allow us to handle parameters correctly

        let result = self.getter().compute_next_labelling(
            &ISubpropertyNext {
                parent: None,
                universal: true,
                inner: 0,
                display: String::new(),
            },
            NodeId::ROOT,
        )?;

        trace!(
            "Computed interpretation of {:?}",
            self.property_checker.property
        );

        Ok(result.value.valuation())
    }

    pub(super) fn compute_inner(&mut self) -> Result<(), ExecError> {
        self.current_time = 0;
        self.next_computation_index = 0;
        self.calmable_fixed_points.clear();
        self.num_fixed_point_computations = 0;
        self.num_fixed_point_iterations = 0;
        self.update_labelling(0, true)?;
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
        is_child: bool,
    ) -> Result<BTreeMap<StateId, TimedCheckValue>, ExecError> {
        let subproperty_entry = self
            .property_checker
            .property
            .subproperty_entry(subproperty_index)
            .clone();

        trace!(
            "Updating subproperty {} labelling: {:?}",
            subproperty_index,
            subproperty_entry
        );

        let updated = if is_child {
            match &subproperty_entry {
                ISubproperty::Func(op) => self.update_func(op)?,
                ISubproperty::Next(op) => self.update_next_labelling(op)?,
                ISubproperty::FixedPoint(op) => {
                    self.update_fixed_point_op(subproperty_index, op)?
                }
            }
        } else {
            let ISubproperty::FixedPoint(_) = &subproperty_entry else {
                panic!("Non-child subproperty should be a fixed point");
            };

            self.update_fixed_variable(subproperty_index)?
        };

        trace!(
            "Updated labelling of subproperty {} in time {}: {:?}",
            subproperty_index,
            self.current_time,
            updated
        );

        Ok(updated)
    }
}
