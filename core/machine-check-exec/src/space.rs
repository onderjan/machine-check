use std::{collections::BTreeSet, num::NonZeroUsize, ops::Shr, rc::Rc};

use bimap::BiMap;
use machine_check_common::StateId;
use mck::{
    abstr::{Input, State},
    concr,
};
use petgraph::{prelude::GraphMap, Directed};

pub struct Edge<AI> {
    pub representative_input: AI,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeId(Option<NonZeroUsize>);

impl NodeId {
    pub const START: NodeId = NodeId(None);
}

impl From<StateId> for NodeId {
    fn from(state_id: StateId) -> Self {
        NodeId(Some(state_id.0))
    }
}

impl TryFrom<NodeId> for StateId {
    type Error = ();

    fn try_from(value: NodeId) -> Result<Self, ()> {
        match value.0 {
            Some(id) => Ok(StateId(id)),
            None => Err(()),
        }
    }
}

pub struct Space<I: Input, S: State> {
    node_graph: GraphMap<NodeId, Edge<I>, Directed>,
    state_map: BiMap<StateId, Rc<S>>,
    num_states_for_sweep: usize,
    next_state_id: StateId,
}

impl<I: Input, S: State> Space<I, S> {
    pub fn new() -> Self {
        Self {
            node_graph: GraphMap::new(),
            state_map: BiMap::new(),
            num_states_for_sweep: 32,
            next_state_id: StateId(NonZeroUsize::MIN),
        }
    }

    pub fn get_state_by_id(&self, state_id: StateId) -> &S {
        self.state_map
            .get_by_left(&state_id)
            .expect("State should be in state map")
            .as_ref()
    }

    pub fn remove_outgoing_edges(&mut self, node_id: NodeId) {
        let direct_successor_indices: Vec<_> = self
            .node_graph
            .neighbors_directed(node_id, petgraph::Direction::Outgoing)
            .collect();
        for direct_successor_id in direct_successor_indices {
            self.node_graph.remove_edge(node_id, direct_successor_id);
        }
    }

    pub fn add_step(
        &mut self,
        current_node: NodeId,
        next_state: S,
        representative_input: &I,
    ) -> (StateId, bool) {
        let (next_state_id, inserted) = self.add_state(next_state);
        self.add_edge(current_node, next_state_id.into(), representative_input);
        (next_state_id, inserted)
    }

    fn add_state(&mut self, state: S) -> (StateId, bool) {
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
            match self.next_state_id.0.checked_add(1) {
                Some(result) => self.next_state_id.0 = result,
                None => panic!("Number of states does not fit in usize"),
            }
            state_id
        };

        if !self.node_graph.contains_node(state_id.into()) {
            // insert to graph
            self.node_graph.add_node(state_id.into());
            // state inserted
            (state_id, true)
        } else {
            // already in the graph, not inserted
            (state_id, false)
        }
    }

    fn add_edge(&mut self, from: NodeId, to: NodeId, input: &I) {
        if self.node_graph.contains_edge(from, to) {
            // do nothing
            return;
        }
        self.node_graph.add_edge(
            from,
            to,
            Edge {
                representative_input: input.clone(),
            },
        );
    }

    pub fn get_representative_input(&self, head: NodeId, tail: StateId) -> &I {
        &self
            .node_graph
            .edge_weight(head, tail.into())
            .expect("Edge should be present in graph")
            .representative_input
    }

    pub fn direct_predecessor_iter(&self, node_id: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        self.node_graph
            .neighbors_directed(node_id, petgraph::Direction::Incoming)
    }

    pub fn direct_successor_iter(&self, node_id: NodeId) -> impl Iterator<Item = StateId> + '_ {
        // successors are always states
        self.node_graph
            .neighbors_directed(node_id, petgraph::Direction::Outgoing)
            .map(|successor_id| StateId::try_from(successor_id).unwrap())
    }

    pub fn initial_iter(&self) -> impl Iterator<Item = StateId> + '_ {
        self.direct_successor_iter(NodeId::START)
    }

    pub fn num_states(&self) -> usize {
        self.state_map.len()
    }

    pub fn state_id_iter(&self) -> impl Iterator<Item = StateId> + '_ {
        self.state_map.left_values().cloned()
    }

    pub fn labelled_iter<'a>(
        &'a self,
        name: &'a str,
        complementary: bool,
        optimistic: bool,
    ) -> impl Iterator<Item = Result<StateId, ()>> + 'a {
        self.state_map.iter().filter_map(move |(state_id, state)| {
            if let Some(labelling) = state.get(name) {
                let labelled = match labelling.concrete_value() {
                    Some(concrete_value) => {
                        // negate if necessary
                        let is_true = concrete_value != concr::Bitvector::new(0);
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
                    Some(Ok(*state_id))
                } else {
                    None
                }
            } else {
                Some(Err(()))
            }
        })
    }

    pub fn labelled_nontrivial_scc_indices(
        &self,
        labelled: &BTreeSet<StateId>,
    ) -> BTreeSet<StateId> {
        // construct a new state graph that only contains labelled vertices and transitions between them
        let mut labelled_graph = GraphMap::<StateId, (), Directed>::new();

        for labelled_id in labelled.iter().cloned() {
            labelled_graph.add_node(labelled_id);
            for direct_successor_id in self.direct_successor_iter(labelled_id.into()) {
                labelled_graph.add_edge(labelled_id, direct_successor_id, ());
            }
        }

        // get out the indices in trivial SCC
        let sccs = petgraph::algo::tarjan_scc(&labelled_graph);
        let mut result = BTreeSet::new();
        for scc in sccs {
            if scc.len() == 1 {
                let state_id = scc[0];
                if !labelled_graph.contains_edge(state_id, state_id) {
                    // trivial SCC, do not add to result
                    break;
                }
            }
            // we only labelled states, so they must be
            result.extend(scc.into_iter());
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
        let mut stack = Vec::<NodeId>::new();
        stack.push(NodeId::START);
        while let Some(node_id) = stack.pop() {
            if let Ok(state_id) = StateId::try_from(node_id) {
                if !unmarked.remove(&state_id) {
                    // already was unmarked
                    continue;
                }
            }
            // go on to direct successors
            for direct_successor_id in self.direct_successor_iter(node_id) {
                stack.push(direct_successor_id.into());
            }
        }
        // only unreachable nodes are unmarked, remove them from state map and graph
        for unmarked_id in unmarked {
            self.state_map.remove_by_left(&unmarked_id);
            self.node_graph.remove_node(unmarked_id.into());
        }
        // update the number of states for sweep as 3/2 of current number of states and at least the previous amount
        self.num_states_for_sweep = self
            .state_map
            .len()
            .saturating_mul(3)
            .shr(1usize)
            .max(self.num_states_for_sweep);
    }

    pub fn contains_state_id(&self, id: StateId) -> bool {
        self.state_map.contains_left(&id)
    }
}
