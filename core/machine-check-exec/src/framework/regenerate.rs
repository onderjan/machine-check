use std::collections::BTreeSet;
use std::collections::VecDeque;

use machine_check_common::NodeId;
use machine_check_common::StateId;
use mck::abstr::Machine as AbstrMachine;
use mck::concr::FullMachine;
use mck::misc::Meta;
use mck::refin::Refine;
use mck::refin::{self};

use crate::RefinInput;

impl<M: FullMachine> super::Framework<M> {
    /// Regenerates the state space from a given node, keeping its other parts. Returns whether the state space changed.
    pub(super) fn regenerate(&mut self, from_node_id: NodeId) -> bool {
        let default_input_precision = &self.default_input_precision;
        let default_step_precision = &self.default_step_precision;

        let mut queue = VecDeque::new();

        // clear the step from the initial node so it is processed
        self.work_state.space.clear_step(from_node_id);
        queue.push_back(from_node_id);

        let mut something_changed = false;

        let mut new_states = BTreeSet::new();
        let mut changed_successors = BTreeSet::new();

        // construct state space by breadth-first search
        while let Some(node_id) = queue.pop_front() {
            // if it has already been processed, continue
            let current_has_direct_successor = self
                .work_state
                .space
                .direct_successor_iter(node_id)
                .next()
                .is_some();
            if current_has_direct_successor {
                continue;
            }

            self.work_state.num_generated_states += 1;
            // remove outgoing edges
            let removed_direct_successors = self.work_state.space.clear_step(node_id);

            // prepare precision
            let input_precision: RefinInput<M> = self.work_state.input_precision.get(
                &self.work_state.space,
                node_id,
                default_input_precision,
            );
            let step_precision = self.work_state.step_precision.get(
                &self.work_state.space,
                node_id,
                default_step_precision,
            );

            // get current state, none if we are at start node
            let current_state = if let Ok(state_id) = StateId::try_from(node_id) {
                Some(self.work_state.space.state_data(state_id).clone())
            } else {
                None
            };

            // generate direct successors
            for input in input_precision.into_proto_iter() {
                // TODO param
                let param =
                    <<M as FullMachine>::Refin as refin::Machine<M>>::Param::clean().proto_first();

                // compute the next state
                let mut next_state = {
                    if let Some(current_state) = &current_state {
                        M::Abstr::next(&self.abstract_system, &current_state.result, &input, &param)
                    } else {
                        M::Abstr::init(&self.abstract_system, &input, &param)
                    }
                };

                // apply decay
                step_precision.force_decay(&mut next_state);

                // add the step to the state space
                self.work_state.num_generated_transitions += 1;
                let (next_state_index, inserted) =
                    self.work_state.space.add_step(node_id, next_state, &input);

                if inserted {
                    new_states.insert(next_state_index);
                }

                // add the tail to the queue if it has no direct successors yet
                let next_has_direct_successor = self
                    .work_state
                    .space
                    .direct_successor_iter(next_state_index.into())
                    .next()
                    .is_some();

                if !next_has_direct_successor {
                    // add to queue
                    queue.push_back(next_state_index.into());
                }
            }

            // compare sets of node ids
            let direct_successors: BTreeSet<StateId> = self
                .work_state
                .space
                .direct_successor_iter(node_id)
                .collect();

            let node_changed = direct_successors != removed_direct_successors;

            if node_changed {
                if let Ok(state_id) = StateId::try_from(node_id) {
                    changed_successors.insert(state_id);
                }
            }

            // make sure changed is true if the target nodes are different from the removed ones
            // ignore the edges changing, currently only used for representative inputs
            // which has no impact on verification
            if !something_changed {
                something_changed = node_changed;
            }
        }

        self.work_state.checker.declare_regeneration(
            &self.work_state.space,
            &new_states,
            &changed_successors,
        );

        // Each node now should have at least one direct successor.
        // Assert it to be sure.
        self.space().assert_left_total();

        something_changed
    }
}
