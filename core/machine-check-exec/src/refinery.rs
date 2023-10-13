use std::collections::VecDeque;
use std::marker::PhantomData;

use machine_check_common::{Culprit, ExecError};
use machine_check_common::{ExecStats, StateId};
use mck::abstr;
use mck::refin::{self};

use crate::proposition::Proposition;
use crate::space::NodeId;
use crate::{
    model_check::{self},
    precision::Precision,
    space::Space,
};

pub struct Refinery<I: refin::Input, S: refin::State, M: refin::Machine<I, S>> {
    machine: PhantomData<M>,
    precision: Precision<I, S>,
    space: Space<I::Abstract, S::Abstract>,
    num_refinements: usize,
    use_decay: bool,
}

impl<I: refin::Input, S: refin::State, M: refin::Machine<I, S>> Refinery<I, S, M> {
    pub fn new(use_decay: bool) -> Self {
        let mut refinery = Refinery {
            machine: PhantomData,
            precision: Precision::new(),
            space: Space::new(),
            num_refinements: 0,
            use_decay,
        };
        // generate first space
        refinery.regenerate(NodeId::START);
        refinery
    }

    pub fn verify(&mut self, proposition: &Proposition) -> Result<bool, ExecError> {
        // main refinement loop
        loop {
            let result = model_check::check_prop(&self.space, proposition);
            // if verification was incomplete, try to refine the culprit
            let culprit = match result {
                Ok(conclusion) => return Ok(conclusion),
                Err(err) => match err {
                    ExecError::Incomplete(culprit) => culprit,
                    _ => return Err(err),
                },
            };
            if !self.refine(&culprit) {
                // it really is incomplete
                return Err(ExecError::Incomplete(culprit));
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
        }
    }

    fn refine(&mut self, culprit: &Culprit) -> bool {
        self.num_refinements += 1;
        //info!("Refinement number: {}", self.num_refinements);
        // compute marking
        let mut current_state_mark = S::default();
        let mark_bit = current_state_mark.get_mut(&culprit.name).unwrap();
        *mark_bit = refin::Bitvector::new_marked();

        // try increasing precision of the state preceding current mark
        let mut iter = culprit.path.iter().cloned().rev().peekable();

        while let Some(current_state_id) = iter.next() {
            //info!("State mark: {:?}", current_state_mark);
            //assert_ne!(current_state_mark, S::default());

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
                    self.regenerate(previous_node_id);
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

                /*info!("Previous state: {:?}", previous_state);
                info!("Step cur state mark: {:?}", current_state_mark);
                if self.num_refinements == 22 && i == 4 {
                    info!("HERE");
                }*/
                let (new_state_mark, input_mark) =
                    M::next((previous_state, input), current_state_mark);
                //info!("Step new state mark: {:?}", new_state_mark);

                (input_mark, Some(new_state_mark))
            } else {
                // use init function
                //info!("Init");

                // increasing state precision failed, try increasing init precision
                let (input_mark,) = M::init((input,), current_state_mark);
                (input_mark, None)
            };

            let input_precision = self.precision.mut_input(previous_node_id);

            //info!("Input mark: {:?}", input_mark);
            //info!("Input prec: {:?}", input_precision);
            if input_precision.apply_refin(&input_mark) {
                // single mark applied, regenerate
                self.regenerate(previous_node_id);
                return true;
            }
            // mark not applied, continue iteration
            if let Some(new_state_mark) = new_state_mark {
                // update current state mark
                current_state_mark = new_state_mark;
            } else {
                // we already know the iterator will end
                // break early as current_state_mark is moved from
                break;
            }
        }

        // incomplete
        false
    }

    pub fn regenerate(&mut self, from_node_id: NodeId) {
        let mut queue = VecDeque::new();
        queue.push_back(from_node_id);
        // construct state space by breadth-first search
        while let Some(node_id) = queue.pop_front() {
            // remove outgoing edges
            self.space.remove_outgoing_edges(node_id);

            // prepare precision
            let input_precision = self.precision.get_input(node_id);
            let decay_precision = self.precision.get_decay(node_id);

            // get current state, none if we are at start node
            let current_state = if let Ok(state_id) = StateId::try_from(node_id) {
                Some(self.space.get_state_by_id(state_id).clone())
            } else {
                None
            };

            // generate direct successors
            for input in input_precision.into_proto_iter() {
                let mut next_state = {
                    if let Some(current_state) = &current_state {
                        <<M as refin::Machine<I, S>>::Abstract as abstr::Machine<
                            I::Abstract,
                            S::Abstract,
                        >>::next(current_state, &input)
                    } else {
                        <<M as refin::Machine<I, S>>::Abstract as abstr::Machine<
                            I::Abstract,
                            S::Abstract,
                        >>::init(&input)
                    }
                };
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

    pub fn info(&self) -> ExecStats {
        ExecStats {
            num_states: self.space.num_states(),
            num_refinements: self.num_refinements,
        }
    }
}
