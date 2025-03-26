use std::{
    collections::{BTreeSet, HashSet, VecDeque},
    fmt::Debug,
    ops::ControlFlow,
};

use machine_check_common::{NodeId, StateId};
use mck::{abstr, concr::FullMachine, misc::MetaWrap};
use petgraph::{prelude::GraphMap, Directed};

use crate::{state_store::StateStore, AbstrInput, PanicState, WrappedInput};

mod labelling;

pub struct StateSpace<M: FullMachine> {
    store: StateStore<M>,
    graph: StateGraph<M>,
    /// How many graph nodes should be reached for a mark-and-sweep.
    num_graph_nodes_for_sweep: usize,
}

impl<M: FullMachine> Debug for StateSpace<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StateSpace")
            .field("store", &self.store)
            .field("graph", &self.graph)
            .field("num_graph_nodes_for_sweep", &self.num_graph_nodes_for_sweep)
            .finish()
    }
}

impl<M: FullMachine> StateSpace<M> {
    pub fn new() -> Self {
        Self {
            graph: StateGraph::new(),
            store: StateStore::new(),
            num_graph_nodes_for_sweep: 2,
        }
    }

    pub fn num_states(&self) -> usize {
        self.store.num_states()
    }

    pub fn num_transitions(&self) -> usize {
        self.graph.num_transitions()
    }

    pub fn is_valid(&self) -> bool {
        self.graph.num_transitions() > 0
    }

    pub fn state_id_iter(&self) -> impl Iterator<Item = StateId> + '_ {
        self.store.state_id_iter()
    }

    pub fn initial_iter(&self) -> impl Iterator<Item = StateId> + '_ {
        self.graph.initial_iter()
    }

    pub fn direct_predecessor_iter(&self, node_id: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        self.graph.direct_predecessor_iter(node_id)
    }

    pub fn direct_successor_iter(&self, node_id: NodeId) -> impl Iterator<Item = StateId> + '_ {
        self.graph.direct_successor_iter(node_id)
    }

    pub fn breadth_first_search<T>(
        &self,
        result_fn: impl Fn(StateId, &PanicState<M>) -> ControlFlow<T, ()>,
    ) -> Option<T> {
        self.graph.breadth_first_search(|state_id| {
            let state = self.store.state_data(state_id);
            result_fn(state_id, state)
        })
    }

    pub fn make_compact(&mut self) {
        let graph_used_states = self.graph.make_compact();
        self.store.retain_states(&graph_used_states);

        // update the number of nodes for sweep as 3/2 of current number of nodes
        // but at least the previous amount

        let candidate = self.graph.node_count().saturating_mul(3) >> 1usize;

        if candidate > self.num_graph_nodes_for_sweep {
            self.num_graph_nodes_for_sweep = candidate;
        }
    }

    pub fn garbage_collect(&mut self) -> bool {
        if self.graph.node_count() >= self.num_graph_nodes_for_sweep {
            self.make_compact();
            return true;
        }
        false
    }

    pub fn state_data(&self, state_id: StateId) -> &PanicState<M> {
        self.store.state_data(state_id)
    }

    pub fn clear_steps(&mut self, head_id: NodeId) -> BTreeSet<StateId> {
        self.graph.clear_step(head_id)
    }

    pub fn add_step(
        &mut self,
        head_id: NodeId,
        tail_data: PanicState<M>,
        representative_input: &AbstrInput<M>,
    ) -> (bool, StateId) {
        let (added_state, tail_id) = self.store.add_state(tail_data);

        self.graph.add_step(head_id, tail_id, representative_input);
        (added_state, tail_id)
    }

    pub fn representative_input(&self, head_id: NodeId, tail_id: StateId) -> &AbstrInput<M> {
        self.graph.get_representative_input(head_id, tail_id)
    }

    pub fn nodes(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.graph.node_graph.nodes()
    }
}

impl<M: FullMachine> Default for StateSpace<M> {
    fn default() -> Self {
        Self::new()
    }
}

/// Abstract state space graph.
pub struct StateGraph<M: FullMachine> {
    /// Graph of node ids.
    ///
    /// Always contains at least the root node. It can also contain states.
    node_graph: GraphMap<NodeId, Edge<WrappedInput<M>>, Directed>,
}

impl<M: FullMachine> Debug for StateGraph<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StateGraph")
            .field("node_graph", &self.node_graph)
            .finish()
    }
}

// Abstract state space edge.
#[derive(Debug)]
pub struct Edge<AI> {
    // Representative abstract input for finding culprits.
    pub representative_input: AI,
}

impl<M: FullMachine> StateGraph<M> {
    pub fn new() -> Self {
        // make the graph with only the root node
        let mut node_graph = GraphMap::new();
        node_graph.add_node(NodeId::ROOT);
        Self { node_graph }
    }

    pub fn clear_step(&mut self, head_id: NodeId) -> BTreeSet<StateId> {
        let direct_successor_indices: BTreeSet<_> = self.direct_successor_iter(head_id).collect();
        for direct_successor_id in direct_successor_indices.clone() {
            self.node_graph
                .remove_edge(head_id, direct_successor_id.into());
        }
        direct_successor_indices
    }

    pub fn add_step(
        &mut self,
        current_node: NodeId,
        next_state: StateId,
        representative_input: &<M::Abstr as abstr::Machine<M>>::Input,
    ) -> bool {
        let next_node = next_state.into();

        if self.node_graph.contains_edge(current_node, next_node) {
            // no edge was added
            return false;
        }
        // adding edge adds the next node if not already part of the graph
        self.node_graph.add_edge(
            current_node,
            next_node,
            Edge {
                representative_input: MetaWrap(representative_input.clone()),
            },
        );
        // the edge was added
        true
    }

    pub fn get_representative_input(
        &self,
        head: NodeId,
        tail: StateId,
    ) -> &<M::Abstr as abstr::Machine<M>>::Input {
        &self
            .node_graph
            .edge_weight(head, tail.into())
            .expect("Edge should be present in graph")
            .representative_input
            .0
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
        self.direct_successor_iter(NodeId::ROOT)
    }

    pub fn num_transitions(&self) -> usize {
        self.node_graph.edge_count()
    }

    /// Makes the state space compact by removing unreachable states.
    ///
    /// Returns retained (i.e. still reachable) states.
    pub fn make_compact(&mut self) -> BTreeSet<StateId> {
        let mut marked = BTreeSet::new();

        // Discover all reachable states by depth-first search.
        let mut stack = Vec::<NodeId>::new();
        stack.push(NodeId::ROOT);
        while let Some(node_id) = stack.pop() {
            // mark if it is a state
            if let Ok(state_id) = StateId::try_from(node_id) {
                if !marked.insert(state_id) {
                    // was already marked, do not explore further
                    continue;
                }
            }

            // explore direct successors
            for direct_successor_id in self.direct_successor_iter(node_id) {
                stack.push(direct_successor_id.into());
            }
        }

        // Discover all unmarked, i.e. unreachable states.
        let unmarked: BTreeSet<StateId> =
            BTreeSet::from_iter(self.node_graph.nodes().filter_map(|node_id| {
                node_id
                    .try_into()
                    .ok()
                    .filter(|&state_id| !marked.contains(&state_id))
            }));

        for state in unmarked {
            self.node_graph.remove_node(state.into());
        }

        marked
    }

    pub fn breadth_first_search<T>(
        &self,
        result_fn: impl Fn(StateId) -> ControlFlow<T, ()>,
    ) -> Option<T> {
        // find the earliest-occuring definite panic id by breadth-first search
        let mut queue = VecDeque::<NodeId>::new();
        let mut processed = HashSet::<NodeId>::new();
        queue.push_back(NodeId::ROOT);
        while let Some(node_id) = queue.pop_front() {
            if let Ok(state_id) = StateId::try_from(node_id) {
                if let ControlFlow::Break(result) = result_fn(state_id) {
                    return Some(result);
                }
            }
            // go on to direct successors that have not been processed yet
            processed.insert(node_id);
            for direct_successor_id in self.direct_successor_iter(node_id) {
                if !processed.contains(&direct_successor_id.into()) {
                    queue.push_back(direct_successor_id.into());
                }
            }
        }
        None
    }

    fn node_count(&self) -> usize {
        self.node_graph.node_count()
    }
}

impl<M: FullMachine> Default for StateGraph<M> {
    fn default() -> Self {
        Self::new()
    }
}
