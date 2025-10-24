use std::collections::BTreeMap;
use std::ops::ControlFlow;

use log::log_enabled;
use log::trace;
use machine_check_common::check::Conclusion;
use machine_check_common::check::KnownConclusion;
use machine_check_common::iir::description::IMachine;
use machine_check_common::iir::property::IProperty;
use machine_check_common::ExecError;
use machine_check_common::ExecStats;
use machine_check_common::NodeId;
use machine_check_common::ParamValuation;
use machine_check_common::StateId;
use mck::concr::FullMachine;
use work_state::WorkState;

use crate::space::StateSpace;
use crate::Strategy;
use mck::refin::RefinementValue;

mod refine;
mod regenerate;
mod work_state;

/// Three-valued abstraction refinement framework.
pub struct Framework<M: FullMachine> {
    /// Abstract system.
    abstract_system: M::Abstr,

    machine: IMachine,

    /// Default input precision.
    default_input_precision: RefinementValue,

    /// Default parameter precision.
    default_param_precision: RefinementValue,

    /// Default step precision.
    default_step_precision: RefinementValue,

    /// Work state containing the structures that change during verification.
    work_state: WorkState<M>,
}

impl<M: FullMachine> Framework<M> {
    /// Constructs the framework with a given system and strategy.
    pub fn new(abstract_system: M::Abstr, machine: IMachine, strategy: Strategy) -> Self {
        let input = machine.input();
        let param = machine.param();
        let state = machine.state();

        // default the input and param precision to clean (inputs will be refined)
        let (default_input_precision, default_param_precision) = if strategy.naive_inputs {
            (input.dirty_refin(), param.dirty_refin())
        } else {
            (input.clean_refin(), param.clean_refin())
        };

        // default the step precision to dirty (steps will remain non-decayed)
        let default_step_precision = if strategy.use_decay {
            state.clean_refin()
        } else {
            state.dirty_refin()
        };

        // return the framework with empty state space, before any construction
        Framework {
            abstract_system,
            machine,
            default_input_precision,
            default_step_precision,
            default_param_precision,
            work_state: WorkState::new(),
        }
    }

    pub fn verify(&mut self, property: &IProperty) -> Result<KnownConclusion, ExecError> {
        // loop verification steps until some conclusion is reached
        let result = loop {
            match self.step_verification(property) {
                ControlFlow::Continue(()) => {}
                ControlFlow::Break(result) => break result,
            }
        };

        // make compact after verification for nice state space information
        self.work_state.make_compact();

        if log_enabled!(log::Level::Trace) {
            trace!("Verification final space: {:#?}", self.work_state.space);
        }
        result
    }

    pub fn step_verification(
        &mut self,
        property: &IProperty,
    ) -> ControlFlow<Result<KnownConclusion, ExecError>> {
        // if the space is invalid (just after construction), regenerate it
        if !self.work_state.space.is_valid() {
            self.regenerate(NodeId::ROOT);
        } else if let Some(culprit) = self.work_state.culprit.take() {
            // we have a culprit, refine on it
            if let Err(err) = self.refine(&culprit) {
                // the refinement is incomplete
                return ControlFlow::Break(Err(err));
            }
            // run garbage collection
            self.work_state.garbage_collect();
        }

        if log_enabled!(log::Level::Trace) {
            trace!("Model-checking state space: {:#?}", self.work_state.space);
        }

        // perform model-checking
        match self.work_state.checker.check_property(
            &self.work_state.space,
            &self.machine,
            property,
        ) {
            Ok(Conclusion::Known(conclusion)) => {
                // conclude the result
                ControlFlow::Break(Ok(conclusion))
            }
            Ok(Conclusion::Unknown(culprit)) => {
                // we have a new culprit, continue the control flow
                self.work_state.culprit = Some(culprit);
                ControlFlow::Continue(())
            }
            Ok(Conclusion::NotCheckable) => {
                // should never happen, the state space should be valid
                panic!("The state space should be valid after stepping verification");
            }
            Err(err) => {
                // propagate the error
                ControlFlow::Break(Err(err))
            }
        }
    }

    pub fn current_labellings(
        &mut self,
        property: &IProperty,
    ) -> Result<BTreeMap<usize, BTreeMap<StateId, ParamValuation>>, ExecError> {
        self.work_state
            .checker
            .get_labellings(&self.work_state.space, property)
    }

    pub fn current_conclusion(&mut self, property: &IProperty) -> Result<Conclusion, ExecError> {
        self.work_state
            .checker
            .check_property(&self.work_state.space, &self.machine, property)
    }

    pub fn find_panic_string(&mut self) -> Option<&'static str> {
        self.work_state.find_panic_string()
    }

    pub fn reset(&mut self) {
        // reset the work state
        self.work_state = WorkState::new()
    }

    pub fn info(&mut self) -> ExecStats {
        self.work_state.info()
    }

    pub fn space(&self) -> &StateSpace<M> {
        &self.work_state.space
    }

    pub fn make_compact(&mut self) {
        self.work_state.make_compact();
    }

    pub fn abstract_system(&self) -> &M::Abstr {
        &self.abstract_system
    }
    pub fn machine(&self) -> &IMachine {
        &self.machine
    }
}
