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
    init_precision: mark::Input,
    initial_nodes: Vec<usize>,
    state_graph: GraphMap<usize, (), Directed>,
    state_map: BiMap<usize, Rc<State>>,
    next_precision_map: HashMap<usize, mark::Input>,
}

impl Space {
    fn unknown_input() -> Input {
        Input::default()
    }

    pub fn new() -> Self {
        let mut space = Self {
            init_precision: mark::Input::default(),
            initial_nodes: vec![],
            state_graph: GraphMap::new(),
            state_map: BiMap::new(),
            next_precision_map: HashMap::new(),
        };
        space.generate();
        space
    }

    pub fn generate(&mut self) {
        println!("Generating state space...");
        // clear initial nodes and graph but leave map and precision as-is
        self.initial_nodes.clear();
        self.state_graph.clear();
        // generate initial states
        let mut input = Possibility::first_possibility(&self.init_precision);
        loop {
            let (initial_state_id, _) = self.add_state(Rc::new(State::init(&input)));
            self.initial_nodes.push(initial_state_id);
            if !Possibility::increment_possibility(&self.init_precision, &mut input) {
                break;
            }
        }

        // construct state space by breadth-first search
        let mut queue = VecDeque::<usize>::new();
        queue.extend(self.initial_nodes.iter());

        while let Some(state_index) = queue.pop_front() {
            let state = self.get_state_by_index(state_index);
            let next_precision = self
                .next_precision_map
                .get(&state_index)
                .expect("Indexed state should have next precision")
                .clone();

            //println!("State #{}: {:?}", state_index, state);

            // generate next states
            let mut input = Possibility::first_possibility(&next_precision);
            loop {
                let next_state = state.next(&input);

                let (next_state_index, inserted) = self.add_state(Rc::new(next_state));

                self.add_edge(state_index, next_state_index);

                if inserted {
                    // add to queue
                    queue.push_back(next_state_index);
                }

                if !Possibility::increment_possibility(&next_precision, &mut input) {
                    break;
                }
            }
        }
        println!(
            "Generated state space with {} states.",
            self.state_map.len()
        );
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
        println!("Refining...");
        // compute marking
        let mut state_mark: mark::State = mark::State {
            safe: MarkBitvector::new_marked(),
            ..Default::default()
        };
        //println!("State mark: {:?}", state_mark);
        let input = &Self::unknown_input();

        // try increasing next precision

        for state_index in culprit.path.iter().rev() {
            let state = self.get_state_by_index(*state_index);
            let (new_state_mark, input_mark) =
                mark::State::next((state.as_ref(), input), state_mark);
            /*println!(
                "New state mark: {:?}, input mark: {:?}",
                new_state_mark, input_mark
            );*/

            let state_next_precision = self
                .next_precision_map
                .get_mut(state_index)
                .expect("Indexed state should have next precision");

            let mut joined_precision = state_next_precision.clone();
            joined_precision.apply_join(input_mark);
            if state_next_precision != &joined_precision {
                *state_next_precision = joined_precision;
                println!("Refined step precision.");
                /*println!(
                    "Refined next precision of state {:?} to {:?}",
                    *state, *state_next_precision
                );*/
                return Ok(());
            }

            state_mark = new_state_mark;
        }

        // try increasing init precision
        let (input_mark,) = mark::State::init((input,), state_mark);
        let mut joined_precision = self.init_precision.clone();
        joined_precision.apply_join(input_mark);
        if self.init_precision != joined_precision {
            self.init_precision = joined_precision;
            //println!("Refined init precision to {:?}", self.init_precision);
            println!("Refined init precision.");
            return Ok(());
        }

        // no joy
        Err(anyhow::anyhow!("Incomplete refinement"))
    }

    fn model_check(&self) -> ModelCheckResult {
        // check AG[!bad]
        // bfs from initial states
        let mut open = VecDeque::<usize>::new();
        let mut became_open = HashSet::<usize>::new();
        let mut backtrack_map = HashMap::<usize, usize>::new();

        open.extend(self.initial_nodes.iter());
        became_open.extend(self.initial_nodes.iter());

        while let Some(state_index) = open.pop_front() {
            let state = self.get_state_by_index(state_index);

            // check state
            let safe: ThreeValuedBitvector<1> = state.safe;
            let true_bitvector = ThreeValuedBitvector::<1>::new(1);
            let false_bitvector = ThreeValuedBitvector::<1>::new(0);
            /*println!(
                "Safe: {:?}, true: {:?}, false: {:?}",
                safe, true_bitvector, false_bitvector
            );*/

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

    fn add_edge(&mut self, from: usize, to: usize) {
        self.state_graph.add_edge(from, to, ());
    }
}
