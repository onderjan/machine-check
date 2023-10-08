use std::{
    collections::{BTreeSet, HashMap},
    ops::Shr,
    rc::Rc,
};

use bimap::BiMap;
use mck::{AbstractMachine, FieldManipulate, MachineBitvector};
use petgraph::{prelude::GraphMap, Directed};

pub struct Edge<AI> {
    pub representative_input: AI,
}

pub struct Space<AM: AbstractMachine> {
    initial_states: HashMap<usize, Edge<AM::Input>>,
    state_graph: GraphMap<usize, Edge<AM::Input>, Directed>,
    state_map: BiMap<usize, Rc<AM::State>>,
    num_states_for_sweep: usize,
    next_state_id: usize,
}

impl<AM: AbstractMachine> Space<AM> {
    pub fn new() -> Self {
        Self {
            initial_states: HashMap::new(),
            state_graph: GraphMap::new(),
            state_map: BiMap::new(),
            num_states_for_sweep: 32,
            next_state_id: 0,
        }
    }

    pub fn get_state_by_index(&self, state_index: usize) -> &AM::State {
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
        state: AM::State,
        representative_input: &AM::Input,
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
        next_state: AM::State,
        representative_input: &AM::Input,
    ) -> (usize, bool) {
        let (next_state_index, inserted) = self.add_state(next_state);
        self.add_edge(current_state_index, next_state_index, representative_input);
        (next_state_index, inserted)
    }

    fn add_state(&mut self, state: AM::State) -> (usize, bool) {
        let state = Rc::new(state);
        let state_id = if let Some(state_id) = self.state_map.get_by_right(&state) {
            // state already present in state map and consequentially next precision map
            // might not be in state graph
            *state_id
        } else {
            // add state to map
            // since we can remove states, use separate next state id
            let state_id = self.next_state_id;
            self.state_map.insert(state_id, state);
            match self.next_state_id.checked_add(1) {
                Some(result) => self.next_state_id = result,
                None => panic!("Number of state does not fit in usize"),
            }
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

    fn add_edge(&mut self, from: usize, to: usize, input: &AM::Input) {
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

    pub fn get_representative_step_input(&self, head: usize, tail: usize) -> &AM::Input {
        &self
            .state_graph
            .edge_weight(head, tail)
            .expect("Edge should be present in graph")
            .representative_input
    }

    pub fn get_representative_init_input(&self, init_state: usize) -> &AM::Input {
        &self
            .initial_states
            .get(&init_state)
            .expect("State should be present in initial states")
            .representative_input
    }

    pub fn initial_index_iter(&self) -> impl Iterator<Item = usize> + '_ {
        self.initial_states.keys().cloned()
    }

    pub fn direct_predecessor_index_iter(
        &self,
        state_index: usize,
    ) -> impl Iterator<Item = usize> + '_ {
        self.state_graph
            .neighbors_directed(state_index, petgraph::Direction::Incoming)
    }

    pub fn direct_successor_index_iter(
        &self,
        state_index: usize,
    ) -> impl Iterator<Item = usize> + '_ {
        self.state_graph
            .neighbors_directed(state_index, petgraph::Direction::Outgoing)
    }

    pub fn num_states(&self) -> usize {
        self.state_map.len()
    }

    pub fn index_iter(&self) -> impl Iterator<Item = usize> + '_ {
        self.state_map.left_values().cloned()
    }

    pub fn labelled_index_iter<'a>(
        &'a self,
        name: &'a str,
        complementary: bool,
        optimistic: bool,
    ) -> impl Iterator<Item = Result<usize, ()>> + 'a {
        self.state_map
            .iter()
            .filter_map(move |(state_index, state)| {
                if let Some(labelling) = state.get(name) {
                    let labelled = match labelling.concrete_value() {
                        Some(concrete_value) => {
                            // negate if necessary
                            let is_true = concrete_value != MachineBitvector::new(0);
                            if complementary {
                                !is_true
                            } else {
                                is_true
                            }
                        }
                        None => {
                            // never negate here, just consider if it is optimistic
                            // see https://patricegodefroid.github.io/public_psfiles/marktoberdorf2013.pdf
                            optimistic
                        }
                    };
                    if labelled {
                        Some(Ok(*state_index))
                    } else {
                        None
                    }
                } else {
                    Some(Err(()))
                }
            })
    }

    pub fn parents_iter(&self, state_index: usize) -> impl Iterator<Item = usize> + '_ {
        self.state_graph
            .neighbors_directed(state_index, petgraph::Direction::Incoming)
    }

    pub fn labelled_nontrivial_scc_indices(&self, labelled: &BTreeSet<usize>) -> BTreeSet<usize> {
        // construct a new state graph that only contains labelled vertices and transitions between them
        let mut labelled_graph = GraphMap::<usize, (), Directed>::new();

        for labelled_index in labelled.iter().cloned() {
            labelled_graph.add_node(labelled_index);
            for direct_successor_index in self.direct_successor_index_iter(labelled_index) {
                labelled_graph.add_edge(labelled_index, direct_successor_index, ());
            }
        }

        // get out the indices in trivial SCC
        let sccs = petgraph::algo::tarjan_scc(&self.state_graph);
        let mut result = BTreeSet::new();
        for scc in sccs {
            if scc.len() == 1 {
                let state_index = scc[0];
                if !self.state_graph.contains_edge(state_index, state_index) {
                    // trivial SCC, do not add to result
                    break;
                }
            }
            result.extend(scc);
        }
        result
    }

    pub fn garbage_collect(&mut self) -> bool {
        if self.state_map.len() >= self.num_states_for_sweep {
            self.mark_and_sweep();
            return true;
        }
        false
    }

    fn mark_and_sweep(&mut self) {
        // construct a map containing all of the nodes
        let mut unmarked = BTreeSet::from_iter(self.state_map.left_values().cloned());
        // remove all of the reachable nodes by depth-first search
        let mut stack = Vec::from_iter(self.initial_states.keys().cloned());
        while let Some(state_index) = stack.pop() {
            if unmarked.remove(&state_index) {
                // we marked it, go on to direct successors
                for direct_successor_index in self.direct_successor_index_iter(state_index) {
                    stack.push(direct_successor_index);
                }
            }
        }
        // only unreachable nodes are unmarked, remove them from state map and graph
        for unmarked_index in unmarked {
            self.state_map.remove_by_left(&unmarked_index);
            self.state_graph.remove_node(unmarked_index);
        }
        // update the number of states for sweep as 3/2 of current number of states and at least the previous amount
        self.num_states_for_sweep = self
            .state_map
            .len()
            .saturating_mul(3)
            .shr(1usize)
            .max(self.num_states_for_sweep);
    }

    pub fn contains_state_index(&self, index: usize) -> bool {
        self.state_map.contains_left(&index)
    }
}
