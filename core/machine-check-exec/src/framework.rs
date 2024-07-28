use std::collections::VecDeque;

use log::debug;
use log::info;
use log::log_enabled;
use log::trace;
use log::warn;
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
    /// Number of states generated until now.
    num_generated_states: usize,
    /// Number of transitions generated until now.
    num_generated_transitions: usize,
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
            num_generated_states: 0,
            num_generated_transitions: 0,
            use_decay: strategy.use_decay,
        }
    }

    /// Verifies a CTL property.
    ///
    /// First verifies that the system does not panic, if it does, it is an execution error.
    pub fn verify_property(
        &mut self,
        prop: &Option<Proposition>,
        assume_inherent: bool,
    ) -> Result<bool, ExecError> {
        // verify inherent non-panicking of system first
        let never_panic_prop = Proposition::A(PropTemp::G(PropG(Box::new(Proposition::Literal(
            Literal::new(
                String::from("__panic"),
                crate::proposition::ComparisonType::Eq,
                0,
                None,
            ),
        )))));

        let inherent_result = if assume_inherent {
            None
        } else {
            info!("Verifying the inherent property.");
            Some(self.verify_inner(&never_panic_prop, false)?)
        };

        match prop {
            Some(prop) => {
                if let Some(inherent_result) = inherent_result {
                    if !inherent_result {
                        // inherent property does not hold, return error
                        let Some(panic_str) = self.find_panic_string() else {
                            panic!("Panic string should be found");
                        };
                        return Err(ExecError::InherentPanic(String::from(panic_str)));
                    }
                    info!("The inherent property holds, verifying the given property.");
                } else {
                    warn!("Assuming that the inherent property holds. If it does not, the verification result will be unusable.");
                }

                // inherent property holds
                // verify the property, assuming no panic can occur
                self.verify_inner(prop, true)
            }
            None => {
                // ensure that we have verified the inherent property
                // log the panic string if necessary and return the result
                let Some(inherent_result) = inherent_result else {
                    panic!("Cannot perform inherent property verification while assuming it");
                };

                if !inherent_result {
                    let Some(panic_str) = self.find_panic_string() else {
                        panic!("Panic string should be found");
                    };
                    info!(
                        "Inherent property does not hold, panic string: '{}'",
                        panic_str
                    );
                }
                Ok(inherent_result)
            }
        }
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
        self.num_refinements = 0;
        self.num_generated_states = 0;
        self.num_generated_transitions = 0;
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
            current_state_mark.panic = refin::Bitvector::dirty();
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
        // store the input precision refinements so that the oldest input can be refined first
        let mut input_precision_refinement: Option<(
            NodeId,
            <<M as FullMachine>::Refin as mck::refin::Machine<M>>::Input,
        )> = None;

        while let Some(current_state_id) = iter.next() {
            let previous_node_id = match iter.peek() {
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

            let (input_mark, new_state_mark) = match TryInto::<StateId>::try_into(previous_node_id)
            {
                Ok(previous_state_id) => {
                    trace!(
                        "Finding refinement where original step function was from {:?} to {:?}",
                        previous_state_id,
                        current_state_id
                    );
                    // use step function
                    let previous_state = self.space.get_state_by_id(previous_state_id);

                    if log_enabled!(log::Level::Trace) {
                        trace!("Earlier state: {:?}", previous_state);
                        let current_state = self.space.get_state_by_id(current_state_id);
                        trace!("Later state: {:?}", current_state);
                        trace!("Later mark: {:?}", current_state_mark);
                    }

                    // the previous state must definitely be non-panicking
                    let previous_state = &previous_state.result;

                    let (_refinement_machine, new_state_mark, input_mark) = M::Refin::next(
                        (&self.abstract_system, previous_state, input),
                        current_state_mark,
                    );

                    (input_mark, Some(new_state_mark))
                }
                Err(_) => {
                    trace!(
                        "Finding refinement where original init function was to {:?}",
                        current_state_id
                    );

                    if log_enabled!(log::Level::Trace) {
                        let current_state = self.space.get_state_by_id(current_state_id);
                        trace!("Later state: {:?}", current_state);
                        trace!("Later mark: {:?}", current_state_mark);
                    }
                    // the current state was generated by the init function
                    let (_refinement_machine, input_mark) =
                        M::Refin::init((&self.abstract_system, input), current_state_mark);
                    (input_mark, None)
                }
            };

            let mut input_precision = self.precision.get_input(previous_node_id);

            trace!("Input mark: {:?}", input_mark);

            if input_precision.apply_refin(&input_mark) {
                // refinement can be applied to input precision, store it
                if log_enabled!(log::Level::Trace) {
                    if let Ok(previous_state_id) = previous_node_id.try_into() {
                        trace!(
                            "Step candidate id: {:?} node: {:?}, input mark: {:?}",
                            previous_state_id,
                            self.space.get_state_by_id(previous_state_id),
                            input_mark
                        );
                    } else {
                        trace!("Init candidate input mark: {:?}", input_mark);
                    }
                }

                // decide if we should replace refinement
                let replace_refinement =
                    if let Some(ref input_precision_refinement) = input_precision_refinement {
                        trace!(
                            "Candidate importance: {}, refinement importance: {}",
                            input_precision.importance(),
                            input_precision_refinement.1.importance()
                        );
                        input_precision.importance() >= input_precision_refinement.1.importance()
                    } else {
                        true
                    };

                if replace_refinement {
                    trace!(
                        "Replacing refinement with candidate with importance {}: {:?}",
                        input_precision.importance(),
                        input_precision
                    );
                    input_precision_refinement = Some((previous_node_id, input_precision));
                }
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

        // if there is an input precision refinement candidate, apply it
        match input_precision_refinement {
            Some((node_id, refined_input_precision)) => {
                let input_precision_mut = self.precision.mut_input(node_id);
                *input_precision_mut = refined_input_precision;

                // single mark applied, regenerate
                self.regenerate(node_id, assume_no_panic);
                true
            }
            None => {
                // cannot apply any refinement, verification incomplete
                false
            }
        }
    }

    /// Regenerates the state space from a given node, keeping its other parts.
    pub fn regenerate(&mut self, from_node_id: NodeId, assume_no_panic: bool) {
        if log_enabled!(log::Level::Trace) {
            trace!(
                "Regenerating with input precision {:?}",
                self.precision.input_precision()
            );
        }
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
                decay_precision.panic = refin::Bitvector::dirty();
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

                self.num_generated_transitions += 1;
                let (next_state_index, added) = self.space.add_step(node_id, next_state, &input);

                if added {
                    self.num_generated_states += 1;
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
            num_generated_states: self.num_generated_states,
            num_final_states: self.space.num_states(),
            num_generated_transitions: self.num_generated_transitions,
            num_final_transitions: self.space.num_transitions(),
        }
    }
}
