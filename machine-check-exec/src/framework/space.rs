use std::{collections::HashMap, rc::Rc};

use bimap::BiMap;
use petgraph::{prelude::GraphMap, Directed};

use crate::machine::forward::{Input, State};

pub struct Edge {
    pub representative_input: Input,
}

pub struct Space {
    initial_states: HashMap<usize, Edge>,
    state_graph: GraphMap<usize, Edge, Directed>,
    state_map: BiMap<usize, Rc<State>>,
}

impl Space {
    pub fn new() -> Self {
        Self {
            initial_states: HashMap::new(),
            state_graph: GraphMap::new(),
            state_map: BiMap::new(),
        }
    }

    pub fn get_state_by_index(&self, state_index: usize) -> &State {
        self.state_map
            .get_by_left(&state_index)
            .expect("Indexed state should be in state map")
            .as_ref()
    }

    pub fn remove_initial_states(&mut self) {
        self.initial_states.clear();
    }

    pub fn remove_outgoing_edges(&mut self, state_index: usize) {
        let direct_successor_indices: Vec<_> = self
            .state_graph
            .neighbors_directed(state_index, petgraph::Direction::Outgoing)
            .collect();
        for direct_successor_index in direct_successor_indices {
            self.state_graph
                .remove_edge(state_index, direct_successor_index);
        }
    }

    pub fn add_initial_state(
        &mut self,
        state: State,
        representative_input: &Input,
    ) -> (usize, bool) {
        let (initial_state_id, added) = self.add_state(state);
        if !self.initial_states.contains_key(&initial_state_id) {
            self.initial_states
                .entry(initial_state_id)
                .or_insert_with(|| Edge {
                    representative_input: representative_input.clone(),
                });
        }
        (initial_state_id, added)
    }

    pub fn add_step(
        &mut self,
        current_state_index: usize,
        next_state: State,
        representative_input: &Input,
    ) -> (usize, bool) {
        let (next_state_index, inserted) = self.add_state(next_state);
        self.add_edge(current_state_index, next_state_index, representative_input);
        (next_state_index, inserted)
    }

    fn add_state(&mut self, state: State) -> (usize, bool) {
        let state = Rc::new(state);
        let state_id = if let Some(state_id) = self.state_map.get_by_right(&state) {
            // state already present in state map and consequentially next precision map
            // might not be in state graph
            *state_id
        } else {
            // add state to map
            let state_id = self.state_map.len();
            self.state_map.insert(state_id, state);
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
            Edge {
                representative_input: input.clone(),
            },
        );
    }

    pub fn get_representative_step_input(&self, head: usize, tail: usize) -> &Input {
        &self
            .state_graph
            .edge_weight(head, tail)
            .expect("Edge should be present in graph")
            .representative_input
    }

    pub fn get_representative_init_input(&self, init_state: usize) -> &Input {
        &self
            .initial_states
            .get(&init_state)
            .expect("State should be present in initial states")
            .representative_input
    }

    pub fn initial_state_indices_iter(&self) -> impl Iterator<Item = usize> + '_ {
        self.initial_states.keys().cloned()
    }

    pub fn direct_successor_indices_iter(
        &self,
        state_index: usize,
    ) -> impl Iterator<Item = usize> + '_ {
        self.state_graph
            .neighbors_directed(state_index, petgraph::Direction::Outgoing)
    }

    pub(crate) fn num_states(&self) -> usize {
        self.state_map.len()
    }
}
