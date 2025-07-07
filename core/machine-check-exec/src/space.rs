use std::{collections::BTreeSet, fmt::Debug, ops::ControlFlow};

use graph::StateGraph;
use machine_check_common::{NodeId, StateId};
use mck::concr::FullMachine;
use store::StateStore;

use crate::{AbstrInput, AbstrPanicState};

mod graph;
mod labelling;
mod store;

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

    pub fn add_step(
        &mut self,
        head_id: NodeId,
        tail_data: AbstrPanicState<M>,
        representative_input: &AbstrInput<M>,
    ) -> (StateId, bool) {
        let (tail_id, inserted) = self.store.state_id(tail_data);

        self.graph.add_step(head_id, tail_id, representative_input);
        (tail_id, inserted)
    }

    pub fn breadth_first_search<T>(
        &self,
        result_fn: impl Fn(StateId, &AbstrPanicState<M>) -> ControlFlow<T, ()>,
    ) -> Option<T> {
        self.graph.breadth_first_search(|state_id| {
            let state = self.store.state_data(state_id);
            result_fn(state_id, state)
        })
    }

    pub fn should_compact(&mut self) -> bool {
        if self.graph.node_count() >= self.num_graph_nodes_for_sweep {
            return true;
        }
        false
    }

    pub fn make_compact(&mut self, outside_used_state_ids: BTreeSet<StateId>) -> BTreeSet<StateId> {
        let mut states = self.graph.make_compact();

        states.extend(outside_used_state_ids);

        self.store.retain_states(&states);

        // update the number of nodes for sweep as 3/2 of current number of nodes
        // but at least the previous amount

        let candidate = self.graph.node_count().saturating_mul(3) >> 1usize;

        if candidate > self.num_graph_nodes_for_sweep {
            self.num_graph_nodes_for_sweep = candidate;
        }
        states
    }

    pub fn state_id(&mut self, state_data: AbstrPanicState<M>) -> StateId {
        // TODO: this is a dangerous thing with garbage collection
        self.store.state_id(state_data).0
    }

    pub fn state_data(&self, state_id: StateId) -> &AbstrPanicState<M> {
        self.store.state_data(state_id)
    }

    pub fn clear_step(&mut self, head_id: NodeId) -> BTreeSet<StateId> {
        self.graph.clear_step(head_id)
    }

    pub fn representative_input(&self, head_id: NodeId, tail_id: StateId) -> &AbstrInput<M> {
        self.graph.representative_input(head_id, tail_id)
    }

    pub fn nodes(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.graph.nodes()
    }

    pub fn states(&self) -> impl Iterator<Item = StateId> + '_ {
        self.nodes().filter_map(|node_id| {
            let Ok(state_id) = StateId::try_from(node_id) else {
                return None;
            };
            Some(state_id)
        })
    }

    pub fn num_nodes(&self) -> usize {
        self.graph.node_count()
    }

    pub fn num_transitions(&self) -> usize {
        self.graph.num_transitions()
    }

    pub fn is_valid(&self) -> bool {
        self.graph.num_transitions() > 0
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

    pub fn assert_left_total(&self) {
        self.graph.assert_left_total();
    }
}

impl<M: FullMachine> Default for StateSpace<M> {
    fn default() -> Self {
        Self::new()
    }
}
