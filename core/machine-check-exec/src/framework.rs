use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::ops::ControlFlow;

use log::debug;
use log::log_enabled;
use log::trace;
use machine_check_common::check::Conclusion;
use machine_check_common::check::Culprit;
use machine_check_common::check::PreparedProperty;
use machine_check_common::property::Property;
use machine_check_common::ExecError;
use machine_check_common::ExecStats;
use machine_check_common::NodeId;
use machine_check_common::StateId;
use machine_check_common::ThreeValued;
use mck::abstr;
use mck::concr::FullMachine;
use mck::misc::Meta;
use mck::refin::Manipulatable;
use mck::refin::{self};

use crate::{
    model_check::{self},
    precision::Precision,
    space::Space,
};
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

pub enum VerificationType {
    Inherent,
    Property(Property),
}

/// Three-valued abstraction refinement framework.
pub struct Framework<M: FullMachine> {
    /// Abstract system.
    abstract_system: M::Abstr,
    /// Whether each step output should decay to fully-unknown by default.
    use_decay: bool,

    /// Work state containing the structures that change during verification.
    work_state: WorkState<M>,
}

/// Work state, i.e. the meta-state of the whole verification.
struct WorkState<M: FullMachine> {
    /// Refinement precision.
    precision: Precision<M>,
    /// Current state space.
    space: Space<M>,
    /// Culprit of verification returning unknown.
    culprit: Option<Culprit>,

    /// Number of refinements made until now.
    num_refinements: usize,
    /// Number of states generated until now.
    num_generated_states: usize,
    /// Number of transitions generated until now.
    num_generated_transitions: usize,
}

impl<M: FullMachine> WorkState<M> {
    fn new(naive_inputs: bool) -> Self {
        Self {
            precision: Precision::new(naive_inputs),
            space: Space::new(),
            culprit: None,
            num_refinements: 0,
            num_generated_states: 0,
            num_generated_transitions: 0,
        }
    }

    fn info(&mut self) -> ExecStats {
        ExecStats {
            num_refinements: self.num_refinements,
            num_generated_states: self.num_generated_states,
            num_final_states: self.space.num_states(),
            num_generated_transitions: self.num_generated_transitions,
            num_final_transitions: self.space.num_transitions(),
            inherent_panic_message: self.find_panic_string().map(String::from),
        }
    }

    fn find_panic_string(&mut self) -> Option<&'static str> {
        // TODO: this approach does not work if there are multiple macro invocations
        let panic_id = self.space.find_panic_id()?;
        Some(M::panic_message(panic_id))
    }
}

impl<M: FullMachine> Framework<M> {
    /// Constructs the framework with a given system and strategy.
    pub fn new(
        abstract_system: M::Abstr,
        //verification_type: VerificationType,
        strategy: &Strategy,
    ) -> Self {
        // return the framework with empty state space, before any construction
        Framework {
            abstract_system,
            use_decay: strategy.use_decay,
            work_state: WorkState::new(strategy.naive_inputs),
        }
    }

    pub fn reset(&mut self) {
        // reset the work state
        self.work_state = WorkState::new(self.work_state.precision.naive_inputs())
    }

    pub fn release_abstract_system(self) -> M::Abstr {
        self.abstract_system
    }

    pub fn verify(
        &mut self,
        property: &PreparedProperty,
        assume_inherent: bool,
    ) -> Result<bool, ExecError> {
        // loop verification steps until some conclusion is reached
        let result = loop {
            match self.step_verification(property, assume_inherent) {
                ControlFlow::Continue(()) => {}
                ControlFlow::Break(result) => break result,
            }
        };

        if log_enabled!(log::Level::Debug) {
            if !assume_inherent {
                debug!(
                    "Property checking final space: {:#?}",
                    self.work_state.space
                );
            } else {
                trace!(
                    "Inherent no-panic checking final space: {:#?}",
                    self.work_state.space
                );
            }
        }
        result
    }

    pub fn step_verification(
        &mut self,
        property: &PreparedProperty,
        assume_inherent: bool,
    ) -> ControlFlow<Result<bool, ExecError>> {
        // if the space is empty (just after construction), regenerate it
        if self.work_state.space.is_empty() {
            self.regenerate(assume_inherent, NodeId::ROOT);
        } else if let Some(culprit) = self.work_state.culprit.take() {
            // we have a culprit, refine on it
            if let Err(err) = self.refine(assume_inherent, &culprit) {
                // the refinement is incomplete
                return ControlFlow::Break(Err(err));
            }
            // run garbage collection
            self.garbage_collect();
        }

        if log_enabled!(log::Level::Trace) {
            trace!("Model-checking state space: {:#?}", self.work_state.space);
        }

        // perform model-checking
        match model_check::check_property::<M>(&self.work_state.space, property) {
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

    fn garbage_collect(&mut self) {
        if self.work_state.space.garbage_collect() {
            self.work_state.precision.retain_indices(|node_id| {
                if let Ok(state_id) = StateId::try_from(node_id) {
                    // only retain those states that are contained
                    self.work_state.space.contains_state_id(state_id)
                } else {
                    // always retain start precision
                    true
                }
            });
        }
    }

    /// Refines the precision and the state space given a culprit of unknown verification result.
    fn refine(&mut self, assume_inherent: bool, culprit: &Culprit) -> Result<(), ExecError> {
        // subrefine bits until the state space changes.
        while !self.subrefine(assume_inherent, culprit)? {}
        Ok(())
    }

    /// Refines a single bit. OK result contains whether the state space changed.
    fn subrefine(&mut self, assume_inherent: bool, culprit: &Culprit) -> Result<bool, ExecError> {
        self.work_state.num_refinements += 1;
        // compute marking
        let mut current_state_mark =
            mck::refin::PanicResult::<<M::Refin as refin::Machine<M>>::State>::clean();

        // TODO: rework panic name kludge
        if culprit.atomic_property.left().name() == "__panic" {
            current_state_mark.panic = refin::Bitvector::dirty();
        } else {
            // TODO: mark more adequately
            let manip_mark = current_state_mark
                .result
                .get_mut(culprit.atomic_property.left().name())
                .expect("Culprit mark should be manipulatable");

            let manip_mark = if let Some(index) = culprit.atomic_property.left().index() {
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
                None => NodeId::ROOT,
            };

            if self.use_decay {
                // decay is applied last in forward direction, so we will apply it first
                let decay_precision = self.work_state.precision.mut_decay(previous_node_id);
                //info!("Decay prec: {:?}", decay_precision);
                if decay_precision.apply_refin(&current_state_mark) {
                    // single mark applied to decay, regenerate
                    return Ok(self.regenerate(assume_inherent, previous_node_id));
                }
            }

            let input = self
                .work_state
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
                    let previous_state = self.work_state.space.get_state_by_id(previous_state_id);

                    if log_enabled!(log::Level::Trace) {
                        trace!("Earlier state: {:?}", previous_state);
                        let current_state = self.work_state.space.get_state_by_id(current_state_id);
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
                        let current_state = self.work_state.space.get_state_by_id(current_state_id);
                        trace!("Later state: {:?}", current_state);
                        trace!("Later mark: {:?}", current_state_mark);
                    }
                    // the current state was generated by the init function
                    let (_refinement_machine, input_mark) =
                        M::Refin::init((&self.abstract_system, input), current_state_mark);
                    (input_mark, None)
                }
            };

            let mut input_precision = self.work_state.precision.get_input(previous_node_id);

            trace!("Input mark: {:?}", input_mark);

            if input_precision.apply_refin(&input_mark) {
                // refinement can be applied to input precision, store it
                if log_enabled!(log::Level::Trace) {
                    if let Ok(previous_state_id) = previous_node_id.try_into() {
                        trace!(
                            "Step candidate id: {:?} node: {:?}, input mark: {:?}",
                            previous_state_id,
                            self.work_state.space.get_state_by_id(previous_state_id),
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
                let input_precision_mut = self.work_state.precision.mut_input(node_id);
                *input_precision_mut = refined_input_precision;

                // single mark applied, regenerate
                Ok(self.regenerate(assume_inherent, node_id))
            }
            None => {
                // cannot apply any refinement, verification incomplete
                Err(ExecError::Incomplete)
            }
        }
    }

    /// Regenerates the state space from a given node, keeping its other parts. Returns whether the state space changed.
    pub fn regenerate(&mut self, assume_inherent: bool, from_node_id: NodeId) -> bool {
        if log_enabled!(log::Level::Trace) {
            trace!(
                "Regenerating with input precision {:?}",
                self.work_state.precision.input_precision()
            );
        }
        let mut queue = VecDeque::new();
        queue.push_back(from_node_id);

        let mut changed = false;
        // construct state space by breadth-first search
        while let Some(node_id) = queue.pop_front() {
            // remove outgoing edges
            let removed_direct_successors = self.work_state.space.remove_outgoing_edges(node_id);

            // prepare precision
            let input_precision = self.work_state.precision.get_input(node_id);
            let mut decay_precision = self.work_state.precision.get_decay(node_id);
            if assume_inherent {
                decay_precision.panic = refin::Bitvector::dirty();
            }

            // get current state, none if we are at start node
            let current_state = if let Ok(state_id) = StateId::try_from(node_id) {
                Some(self.work_state.space.get_state_by_id(state_id).clone())
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
                if assume_inherent {
                    next_state.panic = abstr::Bitvector::new(0);
                }

                if self.use_decay {
                    decay_precision.force_decay(&mut next_state);
                }

                self.work_state.num_generated_transitions += 1;
                let (next_state_index, added) =
                    self.work_state.space.add_step(node_id, next_state, &input);

                if added {
                    self.work_state.num_generated_states += 1;
                    // add to queue
                    queue.push_back(next_state_index.into());
                }
            }

            // make sure changed is true if the target nodes are different from the removed ones
            // ignore the edges changing, currently only used for representative inputs
            // which has no impact on verification
            if !changed {
                // compare sets of node ids
                let direct_successors: HashSet<NodeId> = self
                    .work_state
                    .space
                    .direct_successor_iter(node_id)
                    .map(|state_id| state_id.into())
                    .collect();
                let removed_direct_successors: HashSet<NodeId> =
                    HashSet::from_iter(removed_direct_successors.into_iter());
                changed = direct_successors != removed_direct_successors;
            }
        }
        changed
    }

    pub fn check_property_with_labelling(
        &self,
        property: &PreparedProperty,
    ) -> Result<(Conclusion, HashMap<StateId, ThreeValued>), ExecError> {
        model_check::check_property_with_labelling(&self.work_state.space, property)
    }

    pub fn find_panic_string(&mut self) -> Option<&'static str> {
        self.work_state.find_panic_string()
    }

    pub fn info(&mut self) -> ExecStats {
        self.work_state.info()
    }

    pub fn space(&self) -> &Space<M> {
        &self.work_state.space
    }
}
