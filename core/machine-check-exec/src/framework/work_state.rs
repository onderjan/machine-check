use std::ops::ControlFlow;

use machine_check_common::check::Culprit;
use machine_check_common::ExecStats;
use mck::concr::FullMachine;

use crate::precision::Precision;
use crate::space::StateSpace;
use crate::AbstrInput;
use crate::AbstrPanicState;
use crate::RefinInput;
use crate::RefinPanicState;

/// Work state, i.e. the meta-state of the whole verification.
pub struct WorkState<M: FullMachine> {
    /// Refinement precision for inputs (can make inputs more precise).
    pub input_precision: Precision<AbstrInput<M>, RefinInput<M>>,
    /// Refinement precision for steps (can add step decay).
    pub step_precision: Precision<AbstrPanicState<M>, RefinPanicState<M>>,
    /// Current state space.
    pub space: StateSpace<M>,
    /// Culprit of verification returning unknown.
    pub culprit: Option<Culprit>,

    /// Number of refinements made until now.
    pub num_refinements: usize,
    /// Number of states generated until now.
    pub num_generated_states: usize,
    /// Number of transitions generated until now.
    pub num_generated_transitions: usize,
}

impl<M: FullMachine> WorkState<M> {
    pub fn new() -> Self {
        Self {
            input_precision: Precision::new(),
            step_precision: Precision::new(),
            space: StateSpace::new(),
            culprit: None,
            num_refinements: 0,
            num_generated_states: 0,
            num_generated_transitions: 0,
        }
    }

    pub fn info(&mut self) -> ExecStats {
        ExecStats {
            num_refinements: self.num_refinements,
            num_generated_states: self.num_generated_states,
            num_final_states: self.space.num_nodes().saturating_sub(1),
            num_generated_transitions: self.num_generated_transitions,
            num_final_transitions: self.space.num_transitions(),
            inherent_panic_message: self.find_panic_string().map(String::from),
        }
    }

    pub fn find_panic_string(&mut self) -> Option<&'static str> {
        let panic_id = self.space.breadth_first_search(|_, state_data| {
            if let Some(panic_value) = state_data.panic.concrete_value() {
                if panic_value.is_nonzero() {
                    return ControlFlow::Break(panic_value.as_unsigned() as u32);
                }
            }
            ControlFlow::Continue(())
        });

        // TODO: panic_message approach does not work if there are multiple macro invocations
        panic_id.map(|panic_id: u32| M::panic_message(panic_id))
    }
}
