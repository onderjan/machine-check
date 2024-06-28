use std::collections::VecDeque;

use log::debug;
use log::log_enabled;
use log::trace;
use machine_check_common::ExecError;
use machine_check_common::ExecStats;
use mck::abstr;
use mck::concr::FullMachine;
use mck::misc::Meta;
use mck::refin::Manipulatable;
use mck::refin::{self};

use crate::model_check::Conclusion;
use crate::model_check::Culprit;
use crate::proposition::Literal;
use crate::proposition::PropG;
use crate::proposition::PropTemp;
use crate::proposition::Proposition;
use crate::space::NodeId;
use crate::space::StateId;
use crate::{
    model_check::{self},
    precision::Precision,
    space::Space,
};
use mck::abstr::Abstr;
use mck::abstr::Machine as AbstrMachine;
use mck::refin::Machine as RefinMachine;
use mck::refin::Refine;

/// Abstraction and refinement strategy.
pub struct Strategy {
    /// Whether each input should immediately cover only a single concrete input.
    pub naive_inputs: bool,
    /// Whether each step output should decay to fully-unknown by default.
    pub use_decay: bool,
}

/// Three-valued abstraction refinement framework.
pub struct Framework<M: FullMachine> {
    /// Abstract system.
    abstract_system: M::Abstr,
    /// Refinement precision.
    precision: Precision<M>,
    ///
    space: Space<M>,
    /// Number of refinements made until now.
    num_refinements: usize,
    /// Whether each step output should decay to fully-unknown by default.
    pub use_decay: bool,
}

impl<M: FullMachine> Framework<M> {
    /// Constructs the framework with a given system and strategy.
    pub fn new(system: M, strategy: Strategy) -> Self {
        let abstract_system = M::Abstr::from_concrete(system);
        Framework {
            abstract_system,
            precision: Precision::new(strategy.naive_inputs),
            space: Space::new(),
            num_refinements: 0,
            use_decay: strategy.use_decay,
        }
    }

    /// Verifies a CTL property.
    ///
    /// First verifies that the system does not panic, if it does, it is an execution error.
    pub fn verify_property(&mut self, prop: &Proposition) -> Result<bool, ExecError> {
        // verify inherent non-panicking of system first
        let never_panic_prop = Proposition::A(PropTemp::G(PropG(Box::new(Proposition::Literal(
            Literal::new(
                String::from("__panic"),
                crate::proposition::ComparisonType::Eq,
                0,
                None,
            ),
        )))));
        let inherent_never_panic = self.verify_inner(&never_panic_prop, false)?;
        if !inherent_never_panic {
            let Some(panic_string) = self.find_panic_string() else {
                panic!("Panic string should be found");
            };
            // TODO: panic string
            return Err(ExecError::InherentPanic(String::from(panic_string)));
        }

        // verify the property afterwards
        self.verify_inner(prop, true)
    }

    pub fn verify_inner(
        &mut self,
        prop: &Proposition,
        assume_no_panic: bool,
    ) -> Result<bool, ExecError> {
        // completely regenerate
        self.space = Space::new();
        let naive_inputs = self.precision.naive_inputs();
        self.precision = Precision::new(naive_inputs);
        self.regenerate(NodeId::START, assume_no_panic);

        let prepared_prop = model_check::prepare_prop(prop);

        // main refinement loop
        let result = loop {
            if log_enabled!(log::Level::Trace) {
                trace!("State space: {:#?}", self.space);
            }

            let conclusion = model_check::check_prop::<M>(&self.space, &prepared_prop)?;
            // if verification was incomplete, try to refine the culprit
            let culprit = match conclusion {
                Conclusion::Known(conclusion) => break Ok(conclusion),
                Conclusion::Unknown(culprit) => culprit,
            };
            if !self.refine(&culprit, assume_no_panic) {
                // it really is incomplete
                break Err(ExecError::Incomplete);
            }
            if self.space.garbage_collect() {
                self.precision.retain_indices(|node_id| {
                    if let Ok(state_id) = StateId::try_from(node_id) {
                        // only retain those states that are contained
                        self.space.contains_state_id(state_id)
                    } else {
                        // always retain start precision
                        true
                    }
                });
            }
        };

        if log_enabled!(log::Level::Debug) {
            self.space.mark_and_sweep();
            if assume_no_panic {
                debug!("Property checking final space: {:#?}", self.space);
            } else {
                trace!("Inherent no-panic checking final space: {:#?}", self.space);
            }
        }

        result
    }

    /// Refines the precision and the state space given a culprit of unknown verification result.
    fn refine(&mut self, culprit: &Culprit, assume_no_panic: bool) -> bool {
        self.num_refinements += 1;
        // compute marking
        let mut current_state_mark =
            mck::refin::PanicResult::<<M::Refin as refin::Machine<M>>::State>::clean();

        // TODO: rework panic name kludge
        if culprit.literal.name() == "__panic" {
            current_state_mark.panic = refin::Bitvector::new_marked();
        } else {
            // TODO: mark more adequately
            let manip_mark = current_state_mark
                .result
                .get_mut(culprit.literal.name())
                .expect("Culprit mark should be manipulatable");

            let manip_mark = if let Some(index) = culprit.literal.index() {
                let Some(indexed_manip_mark) = manip_mark.index_mut(index) else {
                    panic!("Indexed culprit mark should be indexable");
                };
                indexed_manip_mark
            } else {
                manip_mark
            };
            manip_mark.mark();
        }

        // try increasing precision of the state preceding current mark
        let mut iter = culprit.path.iter().cloned().rev().peekable();

        while let Some(current_state_id) = iter.next() {
            let previous_state_id = iter.peek();
            let previous_node_id = match previous_state_id {
                Some(previous_state_id) => (*previous_state_id).into(),
                None => NodeId::START,
            };

            if self.use_decay {
                // decay is applied last in forward direction, so we will apply it first
                let decay_precision = self.precision.mut_decay(previous_node_id);
                //info!("Decay prec: {:?}", decay_precision);
                if decay_precision.apply_refin(&current_state_mark) {
                    // single mark applied to decay, regenerate
                    self.regenerate(previous_node_id, assume_no_panic);
                    return true;
                }
            }

            let input = self
                .space
                .get_representative_input(previous_node_id, current_state_id);

            let (input_mark, new_state_mark) = if let Some(previous_state_index) = previous_state_id
            {
                // use step function
                let previous_state = self.space.get_state_by_id(*previous_state_index);

                // the previous state must definitely be non-panicking
                let previous_state = &previous_state.result;

                let (_refinement_machine, new_state_mark, input_mark) = M::Refin::next(
                    (&self.abstract_system, previous_state, input),
                    current_state_mark,
                );

                (input_mark, Some(new_state_mark))
            } else {
                // increasing state precision failed, try increasing init precision
                let (_refinement_machine, input_mark) =
                    M::Refin::init((&self.abstract_system, input), current_state_mark);
                (input_mark, None)
            };

            let input_precision = self.precision.mut_input(previous_node_id);

            if input_precision.apply_refin(&input_mark) {
                // single mark applied, regenerate
                self.regenerate(previous_node_id, assume_no_panic);
                return true;
            }
            // mark not applied, continue iteration
            if let Some(new_state_mark) = new_state_mark {
                // update current state mark
                // note that the preceding state could not have panicked
                current_state_mark = mck::refin::PanicResult {
                    panic: refin::Bitvector::new_unmarked(),
                    result: new_state_mark,
                };
            } else {
                // we already know the iterator will end
                // break early as current_state_mark is moved from
                break;
            }
        }

        // incomplete
        false
    }

    /// Regenerates the state space from a given node, keeping its other parts.
    pub fn regenerate(&mut self, from_node_id: NodeId, assume_no_panic: bool) {
        let mut queue = VecDeque::new();
        queue.push_back(from_node_id);
        // construct state space by breadth-first search
        while let Some(node_id) = queue.pop_front() {
            // remove outgoing edges
            self.space.remove_outgoing_edges(node_id);

            // prepare precision
            let input_precision = self.precision.get_input(node_id);
            let mut decay_precision = self.precision.get_decay(node_id);
            if assume_no_panic {
                decay_precision.panic = refin::Bitvector::new_marked();
            }

            // get current state, none if we are at start node
            let current_state = if let Ok(state_id) = StateId::try_from(node_id) {
                let current_state = self.space.get_state_by_id(state_id).clone();

                let mut can_be_panic = true;
                if let Some(panic_value) = current_state.panic.concrete_value() {
                    if panic_value.is_zero() {
                        can_be_panic = false;
                    }
                }
                if can_be_panic {
                    // skip generation from state
                    // loop back to itself instead to retain left-totality
                    self.space
                        .add_loop(state_id, &input_precision.into_proto_iter().nth(0).unwrap());
                    continue;
                }
                Some(current_state)
            } else {
                None
            };

            // generate direct successors
            for input in input_precision.into_proto_iter() {
                let mut next_state = {
                    if let Some(current_state) = &current_state {
                        M::Abstr::next(&self.abstract_system, &current_state.result, &input)
                    } else {
                        M::Abstr::init(&self.abstract_system, &input)
                    }
                };
                if assume_no_panic {
                    next_state.panic = abstr::Bitvector::new(0);
                }

                if self.use_decay {
                    decay_precision.force_decay(&mut next_state);
                }

                let (next_state_index, added) = self.space.add_step(node_id, next_state, &input);

                if added {
                    // add to queue
                    queue.push_back(next_state_index.into());
                }
            }
        }
    }

    fn find_panic_string(&mut self) -> Option<&'static str> {
        // TODO: this approach does not work if there are multiple macro invocations
        let Some(panic_id) = self.space.find_panic_id() else {
            return None;
        };
        Some(M::panic_message(panic_id))
    }

    pub fn info(&self) -> ExecStats {
        ExecStats {
            num_refinements: self.num_refinements,
            num_generated_states: self.space.num_generated_states(),
            num_final_states: self.space.num_states(),
            num_generated_transitions: self.space.num_generated_transitions(),
            num_final_transitions: self.space.num_transitions(),
        }
    }
}
