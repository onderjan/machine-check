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
    pub path: VecDeque<usize>,
}

#[derive(Debug)]
pub enum ModelCheckResult {
    True,
    False,
    Unknown(Culprit),
}

pub enum VerificationInfo {
    Completed(bool),
    Incomplete(Vec<Rc<State>>),
}

pub struct SpaceEdge {
    pub first_input: Input,
}

pub struct Space {
    init_precision: mark::Input,
    initial_states: HashMap<usize, SpaceEdge>,
    state_graph: GraphMap<usize, SpaceEdge, Directed>,
    state_map: BiMap<usize, Rc<State>>,
    next_precision_map: HashMap<usize, mark::Input>,
    pub num_init_refinements: usize,
    pub num_step_refinements: usize,
}

impl Space {
    pub fn new() -> Self {
        let mut space = Self {
            init_precision: mark::Input::default(),
            initial_states: HashMap::new(),
            state_graph: GraphMap::new(),
            state_map: BiMap::new(),
            next_precision_map: HashMap::new(),
            num_init_refinements: 0,
            num_step_refinements: 0,
        };
        space.regenerate_init();
        space
    }

    pub fn regenerate_init(&mut self) {
        self.num_init_refinements += 1;
        // clear initial states
        self.initial_states.clear();

        // regenerate them using init function with init precision
        // remember the states that were actually added
        let mut added_states_queue = VecDeque::new();
        let mut input = Possibility::first_possibility(&self.init_precision);
        loop {
            let (initial_state_id, added) = self.add_state(Rc::new(State::init(&input)));
            if !self.initial_states.contains_key(&initial_state_id) {
                self.initial_states
                    .entry(initial_state_id)
                    .or_insert_with(|| SpaceEdge {
                        first_input: input.clone(),
                    });
            }
            if added {
                added_states_queue.push_back(initial_state_id);
            }

            if !Possibility::increment_possibility(&self.init_precision, &mut input) {
                break;
            }
        }

        // generate every state that was added
        self.regenerate_step(added_states_queue);
    }

    pub fn regenerate_step(&mut self, mut queue: VecDeque<usize>) {
        self.num_step_refinements += 1;
        // construct state space by breadth-first search
        while let Some(state_index) = queue.pop_front() {
            let state = self.get_state_by_index(state_index);
            let next_precision = self
                .next_precision_map
                .get(&state_index)
                .expect("Indexed state should have next precision")
                .clone();

            // remove outgoing edges
            // use a temporary vector to avoid race conditions
            let direct_successor_indices: Vec<_> = self
                .state_graph
                .neighbors_directed(state_index, petgraph::Direction::Outgoing)
                .collect();
            for direct_successor_index in direct_successor_indices {
                self.state_graph
                    .remove_edge(state_index, direct_successor_index);
            }

            // generate direct successors
            let mut input = Possibility::first_possibility(&next_precision);
            loop {
                let next_state = state.next(&input);

                let (next_state_index, inserted) = self.add_state(Rc::new(next_state));

                self.add_edge(state_index, next_state_index, &input);

                if inserted {
                    // add to queue
                    queue.push_back(next_state_index);
                }

                if !Possibility::increment_possibility(&next_precision, &mut input) {
                    break;
                }
            }
        }
    }

    pub fn verify(&mut self) -> anyhow::Result<VerificationInfo> {
        loop {
            let model_check_result = self.model_check();

            let culprit = match model_check_result {
                ModelCheckResult::True => return Ok(VerificationInfo::Completed(true)),
                ModelCheckResult::False => return Ok(VerificationInfo::Completed(false)),
                ModelCheckResult::Unknown(culprit) => culprit,
            };

            if !self.refine(&culprit)? {
                let mut culprit_states = Vec::new();
                for state_index in &culprit.path {
                    let state = self.get_state_by_index(*state_index);
                    culprit_states.push(state);
                }
                return Ok(VerificationInfo::Incomplete(culprit_states));
            }
        }
    }

    fn refine(&mut self, culprit: &Culprit) -> anyhow::Result<bool> {
        assert!(self
            .initial_states
            .contains_key(culprit.path.front().unwrap()));
        // compute marking
        let mut current_state_mark: mark::State = mark::State {
            safe: MarkBitvector::new_marked(),
            ..Default::default()
        };

        // try increasing precision of the state preceding current mark
        let previous_state_iter = culprit.path.iter().rev().skip(1);
        let current_state_iter = culprit.path.iter().rev();
        let iter = previous_state_iter.zip(current_state_iter);

        for (previous_state_index, current_state_index) in iter {
            assert_ne!(current_state_mark, mark::State::default());

            let previous_state = self.get_state_by_index(*previous_state_index);

            let input = &self
                .state_graph
                .edge_weight(*previous_state_index, *current_state_index)
                .unwrap()
                .first_input;
            // step using the previous state as input
            let (new_state_mark, input_mark) =
                mark::State::next((previous_state.as_ref(), input), current_state_mark);

            let previous_state_precision = self
                .next_precision_map
                .get_mut(previous_state_index)
                .expect("Indexed state should have precision");

            let mut joined_precision = previous_state_precision.clone();
            joined_precision.apply_join(input_mark);
            if previous_state_precision != &joined_precision {
                *previous_state_precision = joined_precision;
                // regenerate step from the state
                let mut queue = VecDeque::new();
                queue.push_back(*previous_state_index);
                self.regenerate_step(queue);
                return Ok(true);
            }

            current_state_mark = new_state_mark;
        }

        let init_input = &self
            .initial_states
            .get(culprit.path.front().unwrap())
            .unwrap()
            .first_input;

        // increasing state precision failed, try increasing init precision
        let (input_mark,) = mark::State::init((init_input,), current_state_mark);

        let mut joined_precision = self.init_precision.clone();
        joined_precision.apply_join(input_mark);
        if self.init_precision != joined_precision {
            self.init_precision = joined_precision;
            // regenerate init
            self.regenerate_init();
            return Ok(true);
        }

        // no joy
        Ok(false)
    }

    fn model_check(&self) -> ModelCheckResult {
        // check AG[!bad]
        // bfs from initial states
        let mut open = VecDeque::<usize>::new();
        let mut became_open = HashSet::<usize>::new();
        let mut backtrack_map = HashMap::<usize, usize>::new();

        open.extend(self.initial_states.keys());
        became_open.extend(self.initial_states.keys());

        while let Some(state_index) = open.pop_front() {
            let state = self.get_state_by_index(state_index);

            // check state
            let safe: ThreeValuedBitvector<1> = state.safe;
            let true_bitvector = ThreeValuedBitvector::<1>::new(1);
            let false_bitvector = ThreeValuedBitvector::<1>::new(0);

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
                assert!(self.initial_states.contains_key(&current_index));

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
                .expect("Indexed state should be in state map"),
        )
    }

    fn add_state(&mut self, state: Rc<State>) -> (usize, bool) {
        let state_id = if let Some(state_id) = self.state_map.get_by_right(&state) {
            // state already present in state map and consequentially next precision map
            // might not be in state graph
            *state_id
        } else {
            // add state to map
            // also add precision which is initially imprecise
            let state_id = self.state_map.len();
            self.state_map.insert(state_id, state);
            self.next_precision_map.insert(state_id, Default::default());
            state_id
        };

        if !self.state_graph.contains_node(state_id) {
            // insert to graph
            self.state_graph.add_node(state_id);
            // state inserted
            (state_id, true)
        } else {
            // already in the graph, not inserted
            (state_id, false)
        }
    }

    fn add_edge(&mut self, from: usize, to: usize, input: &Input) {
        if self.state_graph.contains_edge(from, to) {
            // do nothing
            return;
        }
        self.state_graph.add_edge(
            from,
            to,
            SpaceEdge {
                first_input: input.clone(),
            },
        );
    }
}
