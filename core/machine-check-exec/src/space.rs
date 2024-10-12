use std::{
    collections::{BTreeSet, HashSet, VecDeque},
    fmt::Debug,
    num::NonZeroUsize,
    ops::Shr,
    rc::Rc,
};

use bimap::BiMap;
use mck::{
    abstr::{self, PanicResult},
    concr::FullMachine,
    misc::MetaWrap,
};
use petgraph::{prelude::GraphMap, Directed};

mod labelling;
mod state;

pub use state::{NodeId, StateId};

/// Abstract state space.
pub struct Space<M: FullMachine> {
    /// Graph of node ids. Contains the dummy initial node and other states.
    node_graph: GraphMap<NodeId, Edge<WrappedInput<M>>, Directed>,
    /// Bidirectional map from state ids to the states.
    state_map: BiMap<StateId, Rc<WrappedState<M>>>,
    /// How many states should be reached for a mark-and-sweep.
    num_states_for_sweep: usize,
    /// Next state id.
    next_state_id: StateId,
}

// Abstract state space edge.
pub struct Edge<AI> {
    // Representative abstract input for finding culprits.
    pub representative_input: AI,
}
type PanicState<M> = PanicResult<<<M as FullMachine>::Abstr as abstr::Machine<M>>::State>;
type WrappedInput<M> = MetaWrap<<<M as FullMachine>::Abstr as abstr::Machine<M>>::Input>;
type WrappedState<M> = MetaWrap<PanicState<M>>;

impl<M: FullMachine> Space<M> {
    pub fn new() -> Self {
        Self {
            node_graph: GraphMap::new(),
            state_map: BiMap::new(),
            num_states_for_sweep: 32,
            next_state_id: StateId(NonZeroUsize::MIN),
        }
    }

    pub fn get_state_by_id(&self, state_id: StateId) -> &PanicState<M> {
        &self
            .state_map
            .get_by_left(&state_id)
            .expect("State should be in state map")
            .as_ref()
            .0
    }

    pub fn remove_outgoing_edges(&mut self, node_id: NodeId) -> Vec<NodeId> {
        let direct_successor_indices: Vec<_> = self
            .node_graph
            .neighbors_directed(node_id, petgraph::Direction::Outgoing)
            .collect();
        for direct_successor_id in direct_successor_indices.clone() {
            self.node_graph.remove_edge(node_id, direct_successor_id);
        }
        direct_successor_indices
    }

    pub fn add_step(
        &mut self,
        current_node: NodeId,
        next_state: PanicState<M>,
        representative_input: &<M::Abstr as abstr::Machine<M>>::Input,
    ) -> (StateId, bool) {
        let (next_state_id, inserted) = self.add_state(next_state);
        self.add_edge(current_node, next_state_id.into(), representative_input);
        (next_state_id, inserted)
    }

    pub fn add_loop(
        &mut self,
        state_id: StateId,
        representative_input: &<M::Abstr as abstr::Machine<M>>::Input,
    ) {
        self.add_edge(state_id.into(), state_id.into(), representative_input);
    }

    fn add_state(&mut self, state: PanicState<M>) -> (StateId, bool) {
        let state = Rc::new(MetaWrap(state));
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

    fn add_edge(
        &mut self,
        from: NodeId,
        to: NodeId,
        input: &<M::Abstr as abstr::Machine<M>>::Input,
    ) {
        if self.node_graph.contains_edge(from, to) {
            // do nothing
            return;
        }
        self.node_graph.add_edge(
            from,
            to,
            Edge {
                representative_input: MetaWrap(input.clone()),
            },
        );
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
        self.direct_successor_iter(NodeId::START)
    }

    pub fn num_states(&self) -> usize {
        self.state_map.len()
    }

    pub fn num_transitions(&self) -> usize {
        self.node_graph.edge_count()
    }

    pub fn state_id_iter(&self) -> impl Iterator<Item = StateId> + '_ {
        self.state_map.left_values().cloned()
    }

    pub fn garbage_collect(&mut self) -> bool {
        if self.state_map.len() >= self.num_states_for_sweep {
            self.mark_and_sweep();
            return true;
        }
        false
    }

    pub fn mark_and_sweep(&mut self) {
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

    pub fn find_panic_id(&self) -> Option<u32> {
        // find the earliest-occuring definite panic id by breadth-first search
        let mut queue = VecDeque::<NodeId>::new();
        let mut processed = HashSet::<NodeId>::new();
        queue.push_back(NodeId::START);
        while let Some(node_id) = queue.pop_front() {
            if let Ok(state_id) = StateId::try_from(node_id) {
                let state = self.get_state_by_id(state_id);
                if let Some(panic_value) = state.panic.concrete_value() {
                    if panic_value.is_nonzero() {
                        return Some(panic_value.as_unsigned() as u32);
                    }
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

    pub fn is_empty(&self) -> bool {
        self.node_graph.node_count() == 0
    }

    pub fn node_graph(&self) -> &GraphMap<NodeId, Edge<WrappedInput<M>>, Directed> {
        &self.node_graph
    }

    pub fn state_map(&self) -> &BiMap<StateId, Rc<WrappedState<M>>> {
        &self.state_map
    }
}

impl<M: FullMachine> Debug for Space<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // build sorted nodes first
        let mut node_ids: Vec<_> = self.node_graph.nodes().collect();
        node_ids.sort();

        writeln!(f, "Space {{")?;

        // print node states
        for node_id in node_ids {
            let mut outgoing: Vec<_> = self
                .node_graph
                .neighbors_directed(node_id, petgraph::Direction::Outgoing)
                .collect();
            outgoing.sort();

            write!(f, "{:?} (-> {:?}): ", node_id, outgoing)?;

            let state_id = match StateId::try_from(node_id) {
                Ok(state_id) => state_id,
                Err(_) => {
                    writeln!(f, "_,")?;
                    continue;
                }
            };
            let state = &self.state_map.get_by_left(&state_id).unwrap().0;
            state.fmt(f)?;
            writeln!(f)?;
        }

        writeln!(f, "}}")
    }
}

impl<M: FullMachine> Default for Space<M> {
    fn default() -> Self {
        Self::new()
    }
}
