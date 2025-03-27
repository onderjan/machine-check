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

        let node_data = state_space.state_data(state_id);

        // Ensure that the precision is monotone with respect to other states.

        let start = std::time::Instant::now();

        /*for (map_id, map_precision) in self.map.iter() {
            let Ok(map_id) = StateId::try_from(*map_id) else {
                continue;
            };

            let map_data = state_space.state_data(map_id);

            // if the map data covers the node data, we must join the precision
            let phi_result = Phi::phi(node_data.clone(), map_data.clone());

            if phi_result.meta_eq(map_data) {
                // join the precision
                mck::refin::Refine::apply_join(&mut node_precision, map_precision);
            }
        }*/

        // search the graph
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

        /*if (self.map.len() % 10) == 1 {
            log::debug!(
                "GET: Map size: {}, Duration monotone application elapsed now: {:?}, total: {:?}",
                self.map.len(),
                elapsed,
                *get_elapsed
            );
        }*/

        node_precision
    }

    pub fn find_parents_children<M: FullMachine>(
        &mut self,
        state_space: &mut StateSpace<M>,
        ordering_root: StateId,
        target_id: StateId,
    ) -> (BTreeSet<StateId>, BTreeSet<StateId>) {
        let target_data = state_space.state_data(target_id);

        let mut parents = BTreeSet::new();
        let mut children = BTreeSet::new();

        let mut stack = Vec::new();

        let mut resolve_current = |stack: &mut Vec<(StateId, bool)>, current_id: StateId| {
            if current_id == target_id {
                // the same id, i.e. the same state
                return false;
            }

            // compare the state on stack with the target
            let current_data = state_space.state_data(current_id);

            let phi_result = Phi::phi(target_data.clone(), current_data.clone());
            /*println!(
                "Resolving current {}:\n{:?}\nagainst target {}:\n{:?}\nPhi result:\n{:?}",
                current_id, current_data, target_id, target_data, phi_result
            );*/
            if phi_result.meta_eq(current_data) {
                // ancestor, go further on this path and signal it is an ancestor
                stack.push((current_id, true));
                true
            } else if phi_result.meta_eq(target_data) {
                // this must be a descendant as it has a different id
                // as we do not go further on this path, it is a child
                children.insert(current_id);
                false
            } else {
                // other relationship
                // go further on this path as some descendant of stack node still can be a child
                stack.push((current_id, false));
                false
            }
        };

        // resolve the ordering root
        // there is no ancestor of the root, so we ignore the return value
        resolve_current(&mut stack, ordering_root);

        while let Some((stack_id, stack_is_ancestor)) = stack.pop() {
            // compare the stack children
            let direct_successors = BTreeSet::from_iter(self.ordering_graph.neighbors(stack_id));
            let mut target_ancestor_in_direct_successors = false;
            for current_id in direct_successors.into_iter() {
                let current_is_target_ancestor = resolve_current(&mut stack, current_id);
                target_ancestor_in_direct_successors |= current_is_target_ancestor;
            }
            if stack_is_ancestor && !target_ancestor_in_direct_successors {
                // as this is an ancestor and there is no ancestor that is its child, this is a parent
                parents.insert(stack_id);
            }
        }

        (parents, children)
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

        let target_data = state_space.state_data(target_id);

        // if we have no ordering root, just set it and add it to the graph
        let Some(mut ordering_root) = self.ordering_root else {
            self.ordering_root = Some(target_id);
            self.ordering_graph.add_node(target_id);
            return;
        };

        // depth first search for a suitable candidate
        let mut stack = Vec::new();
        stack.push(ordering_root);

        let mut join_state = None;

        while let Some(stack_id) = stack.pop() {
            // look at whether it has a new join
            let stack_data = state_space.state_data(stack_id);

            let phi_result = Phi::phi(target_data.clone(), stack_data.clone());

            if !phi_result.meta_eq(target_data) && !phi_result.meta_eq(stack_data) {
                // add to the map
                let phi_id = state_space.state_id(phi_result);

                // break with this join state
                join_state = Some((phi_id, stack_id == ordering_root));
                break;
            }

            // look at the descendants in the ordering graph
            // the order of iteration can change the structure of the graph
            let ordering_descendants = BTreeSet::from_iter(self.ordering_graph.neighbors(stack_id));
            for descendant in ordering_descendants.into_iter() {
                stack.push(descendant);
            }
        }

        // fix the ordering graph

        // add the join state to the graph first
        if let Some((join_state_id, update_ordering_root)) = join_state {
            //println!("Adding join state {}", join_state_id);
            self.map.insert(join_state_id.into(), default.clone());
            self.add_graph_node(state_space, ordering_root, join_state_id);

            // update the ordering root if necessary
            if update_ordering_root {
                ordering_root = join_state_id;
                self.ordering_root = Some(ordering_root);
            }
        }

        // then add the new state (perhaps with a new ordering root)
        //println!("Adding new state {}", target_id);
        self.add_graph_node(state_space, ordering_root, target_id);

        // regenerate the ordering graph

        /*self.ordering_graph = GraphMap::<StateId, (), Directed>::new();

        let states = BTreeSet::from_iter(
            self.map
                .keys()
                .filter_map(|node_id| StateId::try_from(*node_id).ok()),
        );

        for a in states.iter().cloned() {
            for b in states.iter().cloned() {
                let a_data = state_space.state_data(a);
                let b_data = state_space.state_data(b);

                let phi_result = Phi::phi(a_data.clone(), b_data.clone());

                if phi_result.meta_eq(a_data) {
                    self.ordering_graph.add_edge(a, b, ());
                }
            }
            // reflexive reduction
            self.ordering_graph.remove_edge(a, a);
        }*/

        // transitive reduction
        //let start_transitive_reduction = std::time::Instant::now();

        // additional expensive assertions, normally commented out
        // use them if a bug in the graph is suspected
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
                        panic!("Not transitively reduced: {} -> {} -> {}", a, b, c);
                        //self.ordering_graph.remove_edge(a, c);
                    }
                }
            }
        }*/

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

    fn add_graph_node<M: FullMachine>(
        &mut self,
        state_space: &mut StateSpace<M>,
        ordering_root: StateId,
        target_id: StateId,
    ) {
        // add the node
        self.ordering_graph.add_node(target_id);
        // find the parents and children
        let (parents, children) = self.find_parents_children(state_space, ordering_root, target_id);

        // as we already have the ordering root, the node should have at least one parent or child
        assert!(!parents.is_empty() || !children.is_empty());

        /*println!(
            "Node: {}, parents: {:?}, children: {:?}",
            target_id, parents, children
        );*/
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

        /*println!(
            "Added graph node {}, graph: {:?}",
            target_id, self.ordering_graph
        );*/
    }
}
