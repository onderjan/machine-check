use std::time::Instant;

use log::debug;
use log::log_enabled;
use log::trace;
use machine_check_common::check::Culprit;
use machine_check_common::ExecError;
use machine_check_common::NodeId;
use machine_check_common::StateId;
use mck::concr::FullMachine;
use mck::misc::Meta;
use mck::refin::Machine as RefinMachine;
use mck::refin::Manipulatable;
use mck::refin::Refine;
use mck::refin::{self};

impl<M: FullMachine> super::Framework<M> {
    /// Refines the precision and the state space given a culprit of unknown verification result.
    pub(super) fn refine(&mut self, culprit: &Culprit) -> Result<(), ExecError> {
        // subrefine bits until the state space changes.
        while !self.subrefine(culprit)? {}
        self.work_state.num_refinements += 1;
        Ok(())
    }

    /// Refines a single bit. OK result contains whether the state space changed.
    fn subrefine(&mut self, culprit: &Culprit) -> Result<bool, ExecError> {
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
            current_state_mark.panic = refin::PanicBitvector::dirty();
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
                &self.work_state.space,
                previous_node_id,
                &self.default_step_precision,
            );

            if step_precision.apply_refin(&current_state_mark) {
                // single mark applied to decay, insert it back and regenerate
                self.work_state.step_precision.insert(
                    &mut self.work_state.space,
                    previous_node_id,
                    step_precision,
                    &self.default_step_precision,
                );

                return Ok(self.regenerate(previous_node_id));
            }

            let input = self
                .work_state
                .space
                .representative_input(previous_node_id, current_state_id);

            // TODO param
            let param = <M::Refin as refin::Machine<M>>::Param::clean().proto_first();

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

                    let (_refinement_machine, new_state_mark, input_mark, param_mark) =
                        M::Refin::next(
                            (&self.abstract_system, previous_state, input, &param),
                            current_state_mark,
                        );

                    // TODO do something with param mark

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
                    let (_refinement_machine, input_mark, param_mark) =
                        M::Refin::init((&self.abstract_system, input, &param), current_state_mark);

                    // TODO: use param mark
                    (input_mark, None)
                }
            };

            let mut input_precision = self.work_state.input_precision.get(
                &self.work_state.space,
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
                    panic: refin::PanicBitvector::new_unmarked(),
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
                self.work_state.input_precision.insert(
                    &mut self.work_state.space,
                    node_id,
                    refined_input_precision,
                    &self.default_input_precision,
                );

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
}
