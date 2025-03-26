use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::VecDeque;
use std::ops::ControlFlow;
use std::time::Instant;

use log::debug;
use log::log_enabled;
use log::trace;
use machine_check_common::check::Conclusion;
use machine_check_common::check::Culprit;
use machine_check_common::check::PreparedProperty;
use machine_check_common::ExecError;
use machine_check_common::ExecStats;
use machine_check_common::NodeId;
use machine_check_common::StateId;
use machine_check_common::ThreeValued;
use mck::concr::FullMachine;
use mck::misc::Meta;
use mck::refin::Manipulatable;
use mck::refin::{self};
use work_state::WorkState;

use crate::model_check::{self};
use crate::space::StateSpace;
use crate::RefinInput;
use crate::RefinPanicState;
use crate::Strategy;
use mck::abstr::Machine as AbstrMachine;
use mck::refin::Machine as RefinMachine;
use mck::refin::Refine;

mod work_state;

/// Three-valued abstraction refinement framework.
pub struct Framework<M: FullMachine> {
    /// Abstract system.
    abstract_system: M::Abstr,

    /// Default input precision.
    default_input_precision: RefinInput<M>,

    /// Default step precision.
    default_step_precision: RefinPanicState<M>,

    /// Work state containing the structures that change during verification.
    work_state: WorkState<M>,
}

impl<M: FullMachine> Framework<M> {
    /// Constructs the framework with a given system and strategy.
    pub fn new(abstract_system: M::Abstr, strategy: Strategy) -> Self {
        // default the input precision to clean (inputs will be refined)
        let default_input_precision = if strategy.naive_inputs {
            Refine::dirty()
        } else {
            Refine::clean()
        };

        // default the step precision to dirty (steps will remain non-decayed)
        let default_step_precision = if strategy.use_decay {
            Refine::clean()
        } else {
            Refine::dirty()
        };

        // return the framework with empty state space, before any construction
        Framework {
            abstract_system,
            default_input_precision,
            default_step_precision,
            work_state: WorkState::new(),
        }
    }

    pub fn verify(&mut self, property: &PreparedProperty) -> Result<bool, ExecError> {
        // loop verification steps until some conclusion is reached
        let result = loop {
            match self.step_verification(property) {
                ControlFlow::Continue(()) => {}
                ControlFlow::Break(result) => break result,
            }
        };

        // make compact after verification for nice state space information
        self.make_compact();

        if log_enabled!(log::Level::Trace) {
            trace!("Verification final space: {:#?}", self.work_state.space);
        }
        result
    }

    pub fn step_verification(
        &mut self,
        property: &PreparedProperty,
    ) -> ControlFlow<Result<bool, ExecError>> {
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

    /// Refines the precision and the state space given a culprit of unknown verification result.
    fn refine(&mut self, culprit: &Culprit) -> Result<(), ExecError> {
        // subrefine bits until the state space changes.
        while !self.subrefine(culprit)? {}
        Ok(())
    }

    /// Refines a single bit. OK result contains whether the state space changed.
    fn subrefine(&mut self, culprit: &Culprit) -> Result<bool, ExecError> {
        self.work_state.num_refinements += 1;
        let start_instant = if log_enabled!(log::Level::Debug) {
            Some(Instant::now())
        } else {
            None
        };
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

            // decay is applied last in forward direction, so we will apply it first
            let mut step_precision = self.work_state.step_precision.get(
                self.space(),
                previous_node_id,
                &self.default_step_precision,
            );

            if step_precision.apply_refin(&current_state_mark) {
                // single mark applied to decay, insert it back and regenerate
                self.work_state
                    .step_precision
                    .insert(previous_node_id, step_precision);

                return Ok(self.regenerate(previous_node_id));
            }

            let input = self
                .work_state
                .space
                .representative_input(previous_node_id, current_state_id);

            let (input_mark, new_state_mark) = match TryInto::<StateId>::try_into(previous_node_id)
            {
                Ok(previous_state_id) => {
                    trace!(
                        "Finding refinement where original step function was from {:?} to {:?}",
                        previous_state_id,
                        current_state_id
                    );
                    // use step function
                    let previous_state = self.work_state.space.state_data(previous_state_id);

                    if log_enabled!(log::Level::Trace) {
                        trace!("Earlier state: {:?}", previous_state);
                        let current_state = self.work_state.space.state_data(current_state_id);
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
                        let current_state = self.work_state.space.state_data(current_state_id);
                        trace!("Later state: {:?}", current_state);
                        trace!("Later mark: {:?}", current_state_mark);
                    }
                    // the current state was generated by the init function
                    let (_refinement_machine, input_mark) =
                        M::Refin::init((&self.abstract_system, input), current_state_mark);
                    (input_mark, None)
                }
            };

            let mut input_precision = self.work_state.input_precision.get(
                self.space(),
                previous_node_id,
                &self.default_input_precision,
            );

            trace!("Input mark: {:?}", input_mark);

            if input_precision.apply_refin(&input_mark) {
                // refinement can be applied to input precision, store it
                if log_enabled!(log::Level::Trace) {
                    if let Ok(previous_state_id) = previous_node_id.try_into() {
                        trace!(
                            "Step candidate id: {:?} node: {:?}, input mark: {:?}",
                            previous_state_id,
                            self.work_state.space.state_data(previous_state_id),
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
        let result = match input_precision_refinement {
            Some((node_id, refined_input_precision)) => {
                // single mark applied, insert it back and regenerate
                self.work_state
                    .input_precision
                    .insert(node_id, refined_input_precision);

                Ok(self.regenerate(node_id))
            }
            None => {
                // cannot apply any refinement, verification incomplete
                Err(ExecError::Incomplete)
            }
        };

        if let Some(start_instant) = start_instant {
            debug!(
                "Refinement #{} took {:?}.",
                self.work_state.num_refinements,
                start_instant.elapsed()
            );
        }

        result
    }

    /// Regenerates the state space from a given node, keeping its other parts. Returns whether the state space changed.
    pub fn regenerate(&mut self, from_node_id: NodeId) -> bool {
        let default_input_precision = &self.default_input_precision;
        let default_step_precision = &self.default_step_precision;

        let mut queue = VecDeque::new();
        queue.push_back(from_node_id);

        let mut changed = false;
        // construct state space by breadth-first search
        while let Some(node_id) = queue.pop_front() {
            self.work_state.num_generated_states += 1;
            // remove outgoing edges
            let removed_direct_successors = self.work_state.space.clear_step(node_id);

            // prepare precision
            let input_precision: RefinInput<M> =
                self.work_state
                    .input_precision
                    .get(self.space(), node_id, default_input_precision);
            let step_precision =
                self.work_state
                    .step_precision
                    .get(self.space(), node_id, default_step_precision);

            // get current state, none if we are at start node
            let current_state = if let Ok(state_id) = StateId::try_from(node_id) {
                Some(self.work_state.space.state_data(state_id).clone())
            } else {
                None
            };

            // generate direct successors
            for input in input_precision.into_proto_iter() {
                // compute the next state
                let mut next_state = {
                    if let Some(current_state) = &current_state {
                        M::Abstr::next(&self.abstract_system, &current_state.result, &input)
                    } else {
                        M::Abstr::init(&self.abstract_system, &input)
                    }
                };

                // apply decay
                step_precision.force_decay(&mut next_state);

                // add the step to the state space
                self.work_state.num_generated_transitions += 1;
                let next_state_index = self.work_state.space.add_step(node_id, next_state, &input);

                // add the tail to the queue if it has no direct successors yet
                let has_direct_successor = self
                    .work_state
                    .space
                    .direct_successor_iter(next_state_index.into())
                    .next()
                    .is_some();

                if !has_direct_successor {
                    // add to queue
                    queue.push_back(next_state_index.into());
                }
            }

            // make sure changed is true if the target nodes are different from the removed ones
            // ignore the edges changing, currently only used for representative inputs
            // which has no impact on verification
            if !changed {
                // compare sets of node ids
                let direct_successors: BTreeSet<StateId> = self
                    .work_state
                    .space
                    .direct_successor_iter(node_id)
                    .collect();
                changed = direct_successors != removed_direct_successors;
            }
        }

        // Each node now should have at least one direct successor.
        // Assert it to be sure.
        self.space().assert_left_total();

        changed
    }

    pub fn check_property_with_labelling(
        &self,
        property: &PreparedProperty,
    ) -> Result<(Conclusion, BTreeMap<StateId, ThreeValued>), ExecError> {
        model_check::check_property_with_labelling(&self.work_state.space, property)
    }

    pub fn find_panic_string(&mut self) -> Option<&'static str> {
        self.work_state.find_panic_string()
    }

    pub fn make_compact(&mut self) {
        let precision_used_states = self
            .work_state
            .input_precision
            .used_nodes()
            .chain(self.work_state.step_precision.used_nodes())
            .filter_map(|node_id| StateId::try_from(node_id).ok());
        self.work_state.space.make_compact(precision_used_states);

        // Each node now still should have at least one direct successor.
        // Assert it to be sure.
        self.space().assert_left_total();
    }

    fn garbage_collect(&mut self) {
        if self.work_state.space.should_compact() {
            self.make_compact();
        }
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
}
