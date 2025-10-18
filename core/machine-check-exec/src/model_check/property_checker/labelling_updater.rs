mod fixed_point;
mod local;
mod next;

use std::collections::BTreeMap;

use log::{debug, trace};
use machine_check_common::{
    iir::{ISubproperty, ISubpropertyNext},
    ExecError, NodeId, ParamValuation, StateId,
};
use mck::concr::FullMachine;

use crate::{
    model_check::property_checker::{
        labelling_cacher::LabellingCacher, CheckValue, PropertyChecker,
    },
    space::StateSpace,
};

pub struct LabellingUpdater<'a, M: FullMachine> {
    property_checker: &'a mut PropertyChecker,
    space: &'a StateSpace<M>,

    invalidate: bool,

    //calmable_fixed_points: BTreeSet<usize>,
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
            invalidate: false,
            //calmable_fixed_points: BTreeSet::new(),
            num_fixed_point_computations: 0,
            num_fixed_point_iterations: 0,
        };

        Ok(computer)
    }

    fn getter(&self) -> LabellingCacher<M> {
        LabellingCacher::new(self.property_checker, self.space)
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
            //self.property_checker.incremental_double_check(self.space)?;
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
            },
            NodeId::ROOT,
        )?;

        trace!(
            "Computed interpretation of {:?}",
            self.property_checker.property
        );

        Ok(result.valuation)
    }

    pub(super) fn compute_inner(&mut self) -> Result<(), ExecError> {
        //self.calmable_fixed_points.clear();
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
    ) -> Result<BTreeMap<StateId, CheckValue>, ExecError> {
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

        //eprintln!("Backtrace: {}", std::backtrace::Backtrace::force_capture());

        let updated = match &subproperty_entry {
            ISubproperty::Func(op) => self.update_func(op)?,
            ISubproperty::Next(op) => self.update_next_labelling(op)?,
            ISubproperty::FixedPoint(op) => self.update_fixed_point_op(subproperty_index, op)?,
        };

        trace!(
            "Updated labelling of subproperty {}: {:?}",
            subproperty_index,
            updated
        );

        Ok(updated)
    }
}
