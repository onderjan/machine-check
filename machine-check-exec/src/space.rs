use std::{
    collections::{HashMap, HashSet, VecDeque},
    rc::Rc,
};

use crate::machine::forward::{
    mark::{self},
    Input, State,
};
use bimap::BiMap;
use mck::{mark::Join, Possibility};
use mck::{MarkBitvector, ThreeValuedBitvector};
use petgraph::{prelude::GraphMap, Directed};

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
    input_mark: mark::Input,
    initial_states: Vec<usize>,
    state_map: BiMap<usize, Rc<State>>,
    state_graph: GraphMap<usize, (), Directed>,
}

impl Space {
    fn unknown_input() -> Input {
        Input {
            input_2: ThreeValuedBitvector::new_unknown(),
            input_3: ThreeValuedBitvector::new_unknown(),
        }
    }

    pub fn new() -> Self {
        let mut space = Self {
            input_mark: mark::Input::default(),
            initial_states: vec![],
            state_map: BiMap::new(),
            state_graph: GraphMap::new(),
        };
        space.generate();
        space
    }

    pub fn generate(&mut self) {
        self.initial_states.clear();
        self.state_map.clear();
        self.state_graph.clear();
        // generate initial states
        let mut input = Possibility::first_possibility(&self.input_mark);
        loop {
            let (initial_state_id, _) = self.add_state(Rc::new(State::init(&input)));
            self.initial_states.push(initial_state_id);
            if !Possibility::increment_possibility(&self.input_mark, &mut input) {
                break;
            }
        }

        // construct state space by breadth-first search
        let mut queue = VecDeque::<usize>::new();
        queue.extend(self.initial_states.iter());

        while let Some(state_index) = queue.pop_front() {
            let state = self.get_state_by_index(state_index);
            println!("State #{}: {:?}", state_index, state);

            // generate next states
            let mut input = Possibility::first_possibility(&self.input_mark);
            loop {
                let next_state = state.next(&input);

                let (next_state_index, inserted) = self.add_state(Rc::new(next_state));

                self.add_edge(state_index, next_state_index);

                if inserted {
                    // add to queue
                    queue.push_back(next_state_index);
                }

                if !Possibility::increment_possibility(&self.input_mark, &mut input) {
                    break;
                }
            }
        }
    }

    pub fn verify(&mut self) -> anyhow::Result<bool> {
        loop {
            let model_check_result = self.model_check();

            let culprit = match model_check_result {
                ModelCheckResult::True => return Ok(true),
                ModelCheckResult::False => return Ok(false),
                ModelCheckResult::Unknown(culprit) => culprit,
            };

            self.refine(culprit)?;
            self.generate();
        }
    }

    fn refine(&mut self, culprit: Culprit) -> anyhow::Result<()> {
        // compute marking
        let mut state_mark: mark::State = Default::default();
        state_mark.safe = MarkBitvector::new_marked();
        println!("State mark: {:?}", state_mark);
        let input = &Self::unknown_input();

        for state_index in culprit.path.iter().rev() {
            let state = self.get_state_by_index(*state_index);
            let (new_state_mark, input_mark) =
                mark::State::next((state.as_ref(), input), state_mark);
            println!(
                "New state mark: {:?}, input mark: {:?}",
                new_state_mark, input_mark
            );
            let mut joined_mark = input_mark.clone();
            joined_mark.input_2.apply_join(input_mark.input_2);
            joined_mark.input_3.apply_join(input_mark.input_3);
            if joined_mark != self.input_mark {
                self.input_mark = joined_mark;
                println!("Refined to {:?}", self.input_mark);
                return Ok(());
            }

            state_mark = new_state_mark;
        }

        Err(anyhow::anyhow!("Incomplete refinement"))
    }

    fn model_check(&self) -> ModelCheckResult {
        // check AG[!bad]
        // bfs from initial states
        let mut open = VecDeque::<usize>::new();
        let mut became_open = HashSet::<usize>::new();
        let mut backtrack_map = HashMap::<usize, usize>::new();

        open.extend(self.initial_states.iter());
        became_open.extend(self.initial_states.iter());

        while let Some(state_index) = open.pop_front() {
            let state = self.get_state_by_index(state_index);

            // check state
            let safe: ThreeValuedBitvector<1> = state.safe;
            let true_bitvector = ThreeValuedBitvector::<1>::new(1);
            let false_bitvector = ThreeValuedBitvector::<1>::new(0);
            println!(
                "Safe: {:?}, true: {:?}, false: {:?}",
                safe, true_bitvector, false_bitvector
            );

            if safe == true_bitvector {
                // OK, continue
            } else if safe == false_bitvector {
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

    fn get_state_by_index(&self, state_index: usize) -> Rc<State> {
        Rc::clone(
            self.state_map
                .get_by_left(&state_index)
                .expect("State in queue should be in state map"),
        )
    }

    fn add_state(&mut self, state: Rc<State>) -> (usize, bool) {
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
