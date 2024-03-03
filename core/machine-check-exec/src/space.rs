use std::{
    collections::{BTreeSet, HashSet, VecDeque},
    fmt::Debug,
    num::NonZeroUsize,
    ops::Shr,
    rc::Rc,
};

use bimap::BiMap;
use mck::{
    abstr::{self, ManipField, Manipulatable, PanicResult},
    concr::FullMachine,
    misc::MetaEq,
};
use petgraph::{prelude::GraphMap, Directed};
use std::hash::Hash;

use crate::proposition::{InequalityType, Literal};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct StateId(pub NonZeroUsize);

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeId(Option<NonZeroUsize>);

impl Debug for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Some(id) => write!(f, "{}", id),
            None => write!(f, "0"),
        }
    }
}

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

pub struct Edge<AI> {
    pub representative_input: AI,
}

#[derive(Clone)]
pub struct MetaWrap<E: MetaEq + Debug + Clone + Hash>(E);

impl<E: MetaEq + Debug + Clone + Hash> PartialEq for MetaWrap<E> {
    fn eq(&self, other: &Self) -> bool {
        self.0.meta_eq(&other.0)
    }
}
impl<E: MetaEq + Debug + Clone + Hash> Eq for MetaWrap<E> {}

impl<E: MetaEq + Debug + Clone + Hash> Hash for MetaWrap<E> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

type PanicState<M> = PanicResult<<<M as FullMachine>::Abstr as abstr::Machine<M>>::State>;

type WrappedInput<M> = MetaWrap<<<M as FullMachine>::Abstr as abstr::Machine<M>>::Input>;
type WrappedState<M> = MetaWrap<PanicState<M>>;

pub struct Space<M: FullMachine> {
    node_graph: GraphMap<NodeId, Edge<WrappedInput<M>>, Directed>,
    state_map: BiMap<StateId, Rc<WrappedState<M>>>,
    num_states_for_sweep: usize,
    next_state_id: StateId,
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

    pub fn state_id_iter(&self) -> impl Iterator<Item = StateId> + '_ {
        self.state_map.left_values().cloned()
    }

    pub fn labelled_iter<'a>(
        &'a self,
        literal: &'a Literal,
        optimistic: bool,
    ) -> impl Iterator<Item = Result<StateId, ()>> + 'a {
        self.state_map.iter().filter_map(move |(state_id, state)| {
            let name = literal.name();
            let manip_field = if name == "__panic" {
                let manip_field: &dyn ManipField = &state.0.panic;
                manip_field
            } else {
                match state.0.result.get(name) {
                    Some(manip_field) => manip_field,
                    None => return Some(Err(())),
                }
            };

            let min_unsigned = manip_field.min_unsigned();
            let max_unsigned = manip_field.max_unsigned();
            let right_unsigned = literal.right_number_unsigned();
            let comparison_result = match literal.comparison_type() {
                crate::proposition::ComparisonType::Eq => {
                    if min_unsigned == max_unsigned {
                        Some(min_unsigned == right_unsigned)
                    } else {
                        None
                    }
                }
                crate::proposition::ComparisonType::Neq => {
                    if min_unsigned == max_unsigned {
                        Some(min_unsigned != right_unsigned)
                    } else {
                        None
                    }
                }
                crate::proposition::ComparisonType::Unsigned(inequality_type) => {
                    Self::resolve_inequality(
                        inequality_type,
                        min_unsigned,
                        max_unsigned,
                        right_unsigned,
                    )
                }
                crate::proposition::ComparisonType::Signed(inequality_type) => {
                    let min_signed = manip_field.min_signed();
                    let max_signed = manip_field.max_signed();
                    let right_signed = literal.right_number_signed();
                    Self::resolve_inequality(inequality_type, min_signed, max_signed, right_signed)
                }
            };

            let labelled = match comparison_result {
                Some(comparison_result) => {
                    // negate if necessary
                    if literal.is_complementary() {
                        !comparison_result
                    } else {
                        comparison_result
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
        })
    }

    fn resolve_inequality<T: Ord>(
        inequality_type: &InequalityType,
        min_left: T,
        max_left: T,
        right: T,
    ) -> Option<bool> {
        match inequality_type {
            InequalityType::Lt => {
                if max_left < right {
                    Some(true)
                } else if min_left >= right {
                    Some(false)
                } else {
                    None
                }
            }
            InequalityType::Le => {
                if max_left <= right {
                    Some(true)
                } else if min_left > right {
                    Some(false)
                } else {
                    None
                }
            }
            InequalityType::Gt => {
                if max_left > right {
                    Some(true)
                } else if min_left <= right {
                    Some(false)
                } else {
                    None
                }
            }
            InequalityType::Ge => {
                if max_left <= right {
                    Some(true)
                } else if min_left > right {
                    Some(false)
                } else {
                    None
                }
            }
        }
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
                    // trivial SCC, do not add to result, but continue over other SCCs
                    continue;
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
}
