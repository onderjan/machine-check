use std::{
    collections::{BTreeSet, HashSet, VecDeque},
    fmt::Debug,
    ops::ControlFlow,
};

use machine_check_common::{NodeId, StateId};
use mck::{abstr, concr::FullMachine, misc::MetaWrap};
use petgraph::{prelude::GraphMap, Directed};

use crate::WrappedInput;

/// Abstract state space graph.
pub struct StateGraph<M: FullMachine> {
    /// Graph of node ids.
    ///
    /// Always contains at least the root node. It can also contain states.
    node_graph: GraphMap<NodeId, Edge<WrappedInput<M>>, Directed>,
}

impl<M: FullMachine> Debug for StateGraph<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StateGraph [")?;
        let edge_set = BTreeSet::from_iter(
            self.node_graph
                .all_edges()
                .map(|(head, tail, _edge)| (head, tail)),
        );
        for (head, tail) in edge_set {
            write!(f, "{} -> {}, ", head, tail)?;
        }
        write!(f, "]")
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

        // Each node now still should have at least one direct successor.
        // Assert it to be sure.
        self.assert_left_total();

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
            // if the node was already processed, do not process it again
            if processed.contains(&node_id) {
                continue;
            }

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

    pub fn representative_input(
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

    pub fn direct_predecessor_iter(
        &self,
        node_id: NodeId,
    ) -> impl Iterator<Item = NodeId> + Clone + '_ {
        self.node_graph
            .neighbors_directed(node_id, petgraph::Direction::Incoming)
    }

    pub fn direct_successor_iter(
        &self,
        node_id: NodeId,
    ) -> impl Iterator<Item = StateId> + Clone + '_ {
        // successors are always states
        self.node_graph
            .neighbors_directed(node_id, petgraph::Direction::Outgoing)
            .map(|successor_id| StateId::try_from(successor_id).unwrap())
    }

    pub fn contains_edge(&self, head_id: NodeId, tail_id: StateId) -> bool {
        self.node_graph.contains_edge(head_id, tail_id.into())
    }

    pub fn contains_state(&self, state_id: StateId) -> bool {
        self.node_graph.contains_node(state_id.into())
    }

    pub fn initial_iter(&self) -> impl Iterator<Item = StateId> + Clone + '_ {
        self.direct_successor_iter(NodeId::ROOT)
    }

    pub fn num_transitions(&self) -> usize {
        self.node_graph.edge_count()
    }

    pub fn node_count(&self) -> usize {
        self.node_graph.node_count()
    }

    pub fn nodes(&self) -> impl Iterator<Item = NodeId> + Clone + '_ {
        self.node_graph.nodes()
    }

    pub fn assert_left_total(&self) {
        for node_id in self.nodes() {
            if self.direct_successor_iter(node_id).count() == 0 {
                panic!(
                    "State space should be left-total but node {} has no successor",
                    node_id
                );
            }
        }
    }
}

impl<M: FullMachine> Default for StateGraph<M> {
    fn default() -> Self {
        Self::new()
    }
}
