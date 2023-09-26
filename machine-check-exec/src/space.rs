use std::collections::{HashMap, HashSet, VecDeque};

use bimap::BiMap;
use mck::{MarkBitvector, ThreeValuedBitvector};
use petgraph::{prelude::GraphMap, Directed};

use crate::machine::forward::{mark, Input, State};

#[derive(Debug)]
pub struct Culprit {
    path: VecDeque<usize>,
}

#[derive(Debug)]
pub enum ModelCheckResult {
    True,
    False,
    Unknown(Culprit),
}

pub struct Space {
    initial_states: Vec<usize>,
    state_map: BiMap<usize, State>,
    state_graph: GraphMap<usize, (), Directed>,
}

impl Space {
    fn unknown_input() -> Input {
        Input {
            input_2: ThreeValuedBitvector::unknown(),
            input_3: ThreeValuedBitvector::unknown(),
        }
    }

    pub fn generate() -> Self {
        let unknown_input = Self::unknown_input();

        let mut space = Self {
            initial_states: vec![],
            state_map: BiMap::new(),
            state_graph: GraphMap::new(),
        };

        // generate initial state
        let (initial_state_id, _) = space.add_state(State::init(&unknown_input));
        space.initial_states.push(initial_state_id);

        // construct state space by breadth-first search
        let mut queue = VecDeque::<usize>::new();
        queue.extend(space.initial_states.iter());

        while let Some(state_index) = queue.pop_front() {
            let state: &State = space.get_state_by_index(state_index);
            println!("State #{}: {:?}", state_index, state);

            // generate next state
            let next_state = state.next(&unknown_input);

            let (next_state_index, inserted) = space.add_state(next_state);

            space.add_edge(state_index, next_state_index);

            if inserted {
                // add to queue
                queue.push_back(next_state_index);
            }
        }
        space
    }

    pub fn verify(&self) -> ModelCheckResult {
        let model_check_result = self.model_check();
        let ModelCheckResult::Unknown(culprit) = model_check_result else {
            return model_check_result;
        };

        // compute marking
        let mut state_mark = mark::State {
            state_6: MarkBitvector::new_unmarked(),
            bad_15: MarkBitvector::new_marked(),
        };
        println!("State mark: {:?}", state_mark);
        let input = &Self::unknown_input();

        for state_index in culprit.path.iter().rev() {
            let state = self.get_state_by_index(*state_index);
            let (new_state_mark, input_mark) = mark::State::next(state_mark, state, input);
            println!(
                "New state mark: {:?}, input mark: {:?}",
                new_state_mark, input_mark
            );
            state_mark = new_state_mark;
        }

        // TODO: mark
        todo!();
    }

    pub fn model_check(&self) -> ModelCheckResult {
        // check AG[!bad]
        // bfs from initial states
        let mut open = VecDeque::<usize>::new();
        let mut became_open = HashSet::<usize>::new();
        let mut backtrack_map = HashMap::<usize, usize>::new();

        open.extend(self.initial_states.iter());
        became_open.extend(self.initial_states.iter());

        while let Some(state_index) = open.pop_front() {
            let state: &State = &self.get_state_by_index(state_index);

            // check state
            let bad: ThreeValuedBitvector<1> = state.bad();
            let true_bitvector = ThreeValuedBitvector::<1>::new(1);
            let false_bitvector = ThreeValuedBitvector::<1>::new(0);
            println!(
                "Bad: {:?}, true: {:?}, false: {:?}",
                bad, true_bitvector, false_bitvector
            );

            if bad == false_bitvector {
                // OK, continue
            } else if bad == true_bitvector {
                // definitely not OK
                return ModelCheckResult::False;
            } else {
                // unknown, put together culprit path
                let mut path = VecDeque::<usize>::new();
                path.push_front(state_index);
                let mut current_index = state_index;
                while let Some(prev_index) = backtrack_map.get(&current_index) {
                    current_index = *prev_index;
                    path.push_front(current_index);
                }

                return ModelCheckResult::Unknown(Culprit { path });
            }

            // open direct successors that did not become open yet
            let direct_successor_indices = self
                .state_graph
                .neighbors_directed(state_index, petgraph::Direction::Outgoing);
            for direct_successor_index in direct_successor_indices {
                let inserted = became_open.insert(direct_successor_index);
                if inserted {
                    backtrack_map.insert(direct_successor_index, state_index);
                    open.push_back(direct_successor_index);
                }
            }
        }
        // if no bad result was found, verification succeeds
        ModelCheckResult::True
    }

    pub fn num_states(&self) -> usize {
        self.state_map.len()
    }

    fn get_state_by_index(&self, state_index: usize) -> &State {
        self.state_map
            .get_by_left(&state_index)
            .expect("State in queue should be in state map")
    }

    fn add_state(&mut self, state: State) -> (usize, bool) {
        if let Some(state_id) = self.state_map.get_by_right(&state) {
            // state already present, not inserted
            (*state_id, false)
        } else {
            let state_id = self.state_map.len();
            self.state_map.insert(state_id, state);
            self.state_graph.add_node(state_id);
            // state inserted
            (state_id, true)
        }
    }

    fn add_edge(&mut self, from: usize, to: usize) {
        self.state_graph.add_edge(from, to, ());
    }
}

fn find_culprit() {}
