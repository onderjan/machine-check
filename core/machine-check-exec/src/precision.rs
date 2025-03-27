use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::time::Duration;

use machine_check_common::{NodeId, StateId};
use mck::abstr::Phi;
use mck::concr::FullMachine;
use mck::misc::MetaEq;
use petgraph::prelude::GraphMap;
use petgraph::Directed;

use crate::space::StateSpace;
use crate::AbstrPanicState;

/// Precision configurable by state space nodes.
///
#[derive(Debug)]
pub struct Precision<A, R: Debug + Clone + mck::refin::Refine<A>> {
    map: BTreeMap<NodeId, R>,
    phantom: PhantomData<A>,
    get_elapsed: RefCell<Duration>,
    insert_elapsed: RefCell<Duration>,
    ordering_root: Option<StateId>,
    ordering_graph: GraphMap<StateId, (), Directed>,
}

impl<A, R: Debug + Clone + mck::refin::Refine<A>> Precision<A, R> {
    pub fn new() -> Self {
        Precision {
            map: BTreeMap::new(),
            phantom: PhantomData,
            get_elapsed: RefCell::new(Duration::ZERO),
            insert_elapsed: RefCell::new(Duration::ZERO),
            ordering_root: None,
            ordering_graph: GraphMap::new(),
        }
    }

    pub fn get<M: FullMachine>(
        &self,
        state_space: &StateSpace<M>,
        node_id: NodeId,
        default: &R,
    ) -> R {
        let mut node_precision = match self.map.get(&node_id) {
            Some(input) => input.clone(),
            None => default.clone(),
        };

        let Ok(state_id) = StateId::try_from(node_id) else {
            // root node, just get the value normally
            return node_precision;
        };

        let start = std::time::Instant::now();
        let node_data = state_space.state_data(state_id);

        // Ensure that the precision is monotone with respect to other states.
        // Do this by searching the graph (depth first) for all nodes.

        if let Some(ordering_root) = self.ordering_root {
            let mut stack = Vec::new();
            stack.push(ordering_root);
            let mut opened = HashSet::new();
            let mut important = HashSet::new();

            while let Some(stack_id) = stack.pop() {
                let stack_data = state_space.state_data(stack_id);
                let phi = Phi::phi(node_data.clone(), stack_data.clone());
                if stack_data.meta_eq(&phi) {
                    important.insert(stack_id);
                    for child_id in self.ordering_graph.neighbors(stack_id) {
                        if !opened.contains(&child_id) {
                            opened.insert(child_id);
                            stack.push(stack_id);
                        }
                    }
                }
            }

            for important_id in important {
                let important_precision = self
                    .map
                    .get(&NodeId::from(important_id))
                    .expect("Precision should contain state in ordering graph");
                mck::refin::Refine::apply_join(&mut node_precision, important_precision);
            }
        }

        let elapsed = start.elapsed();
        let mut get_elapsed = self.get_elapsed.borrow_mut();
        *get_elapsed += elapsed;

        node_precision
    }

    pub fn insert<M: FullMachine>(
        &mut self,
        state_space: &mut StateSpace<M>,
        target_id: NodeId,
        value: R,
        default: &R,
    ) {
        let start = std::time::Instant::now();

        // if the node already exists, we do not need to change the graph

        if self.map.insert(target_id, value).is_some() {
            return;
        }

        // if we are changing the root node, we also do not need to change the graph
        let Ok(target_id) = StateId::try_from(target_id) else {
            return;
        };

        let target_data = state_space.state_data(target_id).clone();

        // if we have no ordering root, just set it and add it to the graph
        if self.ordering_root.is_none() {
            self.ordering_root = Some(target_id);
            self.ordering_graph.add_node(target_id);
            return;
        };

        // Find the join candidate, which will serve to provide a hierarchy in the ordering graph.
        let join_state_id = self.join_candidate(state_space, &target_data);
        if let Some(join_state_id) = join_state_id {
            // Add the join state to the graph first, as it may become the new ordering root.
            self.map.insert(join_state_id.into(), default.clone());
            self.add_graph_node(state_space, join_state_id);
        }
        self.add_graph_node(state_space, target_id);

        // We are done, assert graph sanity.
        self.assert_graph_sanity(target_id, join_state_id);

        let elapsed = start.elapsed();
        let get_elapsed = self.get_elapsed.borrow();
        let mut insert_elapsed = self.insert_elapsed.borrow_mut();
        *insert_elapsed += elapsed;
        log::debug!(
            "Inserted to precision with map size {}, retrieval total: {:?}, insertion total: {:?}",
            self.map.len(),
            get_elapsed,
            *insert_elapsed
        );
        /*println!(
            "{:?}",
            Dot::with_config(&self.ordering_graph, &[Config::EdgeNoLabel])
        );*/
    }

    fn join_candidate<M: FullMachine>(
        &mut self,
        state_space: &mut StateSpace<M>,
        target_data: &AbstrPanicState<M>,
    ) -> Option<StateId> {
        // depth first search for a suitable candidate
        let mut stack = Vec::new();
        let Some(ordering_root) = self.ordering_root else {
            panic!("Should have an ordering root when searching for the join candidate");
        };
        stack.push(ordering_root);

        while let Some(stack_id) = stack.pop() {
            // look at whether it has a new join
            let stack_data = state_space.state_data(stack_id);

            let phi_result = Phi::phi(target_data.clone(), stack_data.clone());

            if !phi_result.meta_eq(target_data) && !phi_result.meta_eq(stack_data) {
                // add to the map
                let phi_id = state_space.state_id(phi_result);

                // return this join candidate
                return Some(phi_id);
            }

            // look at the descendants in the ordering graph
            // the order of iteration can change the structure of the graph
            let ordering_descendants = BTreeSet::from_iter(self.ordering_graph.neighbors(stack_id));
            for descendant in ordering_descendants.into_iter() {
                stack.push(descendant);
            }
        }
        // no join candidate found
        None
    }

    fn add_graph_node<M: FullMachine>(
        &mut self,
        state_space: &mut StateSpace<M>,
        target_id: StateId,
    ) {
        let Some(ordering_root) = self.ordering_root else {
            panic!("Should have an ordering root when adding a graph node");
        };

        // if the node already exists, there is no need to add it
        if self.ordering_graph.contains_node(target_id) {
            return;
        }
        // find the parents and children
        let (parents, children) =
            self.find_parents_and_children(state_space, ordering_root, target_id);

        // as we already have the ordering root, the node should have at least one parent or child
        assert!(!parents.is_empty() || !children.is_empty());

        // update the ordering root if the new state has no parents
        if parents.is_empty() {
            self.ordering_root = Some(target_id);
        }

        // the node will be added together with the first edge (at least one exists)
        // add the edges from parents
        for parent in parents.iter().cloned() {
            self.ordering_graph.add_edge(parent, target_id, ());
        }
        // add the edges to children
        for child in children.iter().cloned() {
            self.ordering_graph.add_edge(target_id, child, ());
        }

        // perform a transitive reduction
        // only the transitions between parents and children can be problematic
        for parent in parents {
            for child in children.iter().cloned() {
                self.ordering_graph.remove_edge(parent, child);
            }
        }
    }

    fn find_parents_and_children<M: FullMachine>(
        &mut self,
        state_space: &mut StateSpace<M>,
        ordering_root: StateId,
        target_id: StateId,
    ) -> (BTreeSet<StateId>, BTreeSet<StateId>) {
        let target_data = state_space.state_data(target_id);

        let mut target_parents = BTreeSet::new();
        let mut target_children = BTreeSet::new();

        let mut neither = BTreeSet::new();
        let mut stack = Vec::new();

        // the root has no ancestor
        stack.push((None, ordering_root));

        while let Some((parent_id, current_id)) = stack.pop() {
            if neither.contains(&current_id) {
                // already processed and disqualified, continue
                continue;
            }

            // if we are currently at the target, there is no need to compare, it is neither its parent nor child
            if current_id != target_id {
                // compare the state on stack with the target
                let current_data = state_space.state_data(current_id);

                let phi_result = Phi::phi(target_data.clone(), current_data.clone());
                if phi_result.meta_eq(current_data) {
                    // ancestor
                    // remove the parent of current from target parents and disqualify it
                    // then add current to target parents
                    if let Some(parent_id) = parent_id {
                        target_parents.remove(&parent_id);
                        neither.insert(parent_id);
                    }
                    target_parents.insert(current_id);
                } else if phi_result.meta_eq(target_data) {
                    // descendant
                    // this cannot be the target node as it has a different id
                    // if the parent is a child, remove current from children and disqualify it
                    // otherwise, add it to children
                    let mut can_be_child = true;
                    if let Some(parent_id) = parent_id {
                        if target_children.contains(&parent_id) {
                            target_children.remove(&current_id);
                            neither.insert(current_id);
                            can_be_child = false;
                        }
                    }
                    if can_be_child {
                        target_children.insert(current_id);
                    }
                } else {
                    // neither an ancestor nor a descendant, disqualify it
                    neither.insert(current_id);
                }
            } else {
                // the target is neither its own ancestor nor a descendant
                neither.insert(current_id);
            }

            let direct_successors = BTreeSet::from_iter(self.ordering_graph.neighbors(current_id));
            for direct_successor_id in direct_successors.into_iter() {
                if !neither.contains(&direct_successor_id) {
                    stack.push((Some(current_id), direct_successor_id));
                }
            }
        }

        (target_parents, target_children)
    }

    fn assert_graph_sanity(&self, target_id: StateId, join_state_id: Option<StateId>) {
        // We are done, make cheap assertions of graph sanity.
        // The new ordering root should have no incoming edges.
        if let Some(ordering_root) = self.ordering_root {
            assert!(self
                .ordering_graph
                .neighbors_directed(ordering_root, petgraph::Direction::Incoming)
                .next()
                .is_none());
        } else {
            panic!("Should have an ordering root after adding nodes");
        }

        // ignore the target and join state id if expensive assertions are not enabled
        let _ = (target_id, join_state_id);

        // Expensive assertion that should ensure monotonicity, normally commented out.
        // For every combination of states (a,b) where the join is a,
        // there should be a path from a to b in the graph.

        /*let map_states = BTreeSet::from_iter(
            self.map
                .keys()
                .filter_map(|node_id| StateId::try_from(*node_id).ok()),
        );

        for a_id in map_states.iter().cloned() {
            let paths = petgraph::algo::dijkstra::dijkstra(&self.ordering_graph, a_id, None, |_| 1);
            for b_id in map_states.iter().cloned() {
                let a_data = state_space.state_data(a_id);
                let b_data = state_space.state_data(b_id);
                let phi = Phi::phi(a_data.clone(), b_data.clone());
                if phi.meta_eq(a_data) && !paths.contains_key(&b_id) {
                    panic!(
                        "State {} covers {}, but there is no corresponding path in the graph",
                        a_id, b_id
                    );
                }
            }
        }*/

        // Additional expensive assertions for the graph, normally commented out:
        // - Should not be cyclic.
        //   This also implies antireflexivity.
        // - Should have exactly one connected component.
        // - Should be antitransitive. This is currently only checked for edges,
        //   not paths, Warshall would have to be applied first for that.
        // Uncomment the assertions if a bug in the graph is suspected.
        /*assert!(!petgraph::algo::is_cyclic_directed(&self.ordering_graph));
        assert_eq!(
            petgraph::algo::connected_components(&self.ordering_graph),
            1
        );
        let ordering_graph_nodes = BTreeSet::from_iter(self.ordering_graph.nodes());
        for a in ordering_graph_nodes.iter().cloned() {
            for b in ordering_graph_nodes.iter().cloned() {
                for c in ordering_graph_nodes.iter().cloned() {
                    if self.ordering_graph.contains_edge(a, b)
                        && self.ordering_graph.contains_edge(b, c)
                        && self.ordering_graph.contains_edge(a, c)
                    {
                        println!("Target: {:?}, join: {:?}", target_id, join_state_id);
                        println!("Ordering root: {:?}", self.ordering_root);
                        println!(
                            "{:?}",
                            Dot::with_config(&self.ordering_graph, &[Config::EdgeNoLabel])
                        );
                        panic!("Not transitively reduced: {} -> {} -> {}", a, b, c);
                        //self.ordering_graph.remove_edge(a, c);
                    }
                }
            }
        }*/
    }
}
